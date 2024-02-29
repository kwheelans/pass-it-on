use crate::endpoints::matrix::common::{login, ClientInfo};
use crate::endpoints::matrix::MatrixEndpoint;
use crate::endpoints::Endpoint;
use crate::{Error, LIB_LOG_TARGET};
use futures_util::stream::StreamExt;
use log::{debug, info, warn};
use matrix_sdk::config::SyncSettings;
use matrix_sdk::encryption::verification::{
    format_emojis, Emoji, SasState, SasVerification, Verification, VerificationRequest, VerificationRequestState,
};
use matrix_sdk::ruma::events::key::verification::request::ToDeviceKeyVerificationRequestEvent;
use matrix_sdk::ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent};
use matrix_sdk::Client;
use std::io::Write;

pub(crate) async fn verify_devices(endpoints: &[Box<dyn Endpoint + Send>]) -> Result<(), Error> {
    let mut matrix_endpoints = Vec::new();

    for endpoint in endpoints {
        let downcast_endpoint = endpoint.as_any().downcast_ref::<MatrixEndpoint>();
        match downcast_endpoint {
            None => debug!(target: LIB_LOG_TARGET, "Processed endpoint is not a matrix endpoint"),
            Some(matrix) => matrix_endpoints.push(matrix),
        }
    }
    info!(target: LIB_LOG_TARGET, "Found {} matrix endpoints and will attempt to verify devices", matrix_endpoints.len());
    for endpoint in matrix_endpoints {
        verify_device(endpoint).await?
    }
    Ok(())
}

async fn verify_device(endpoint: &MatrixEndpoint) -> Result<(), Error> {
    let client = login(ClientInfo::from(endpoint)).await?;

    client.add_event_handler(|event: ToDeviceKeyVerificationRequestEvent, client: Client| async move {
        let request = client
            .encryption()
            .get_verification_request(&event.sender, &event.content.transaction_id)
            .await
            .expect("Request object wasn't created");

        tokio::spawn(request_verification_handler(request));
    });

    client.add_event_handler(|event: OriginalSyncRoomMessageEvent, client: Client| async move {
        if let MessageType::VerificationRequest(_) = &event.content.msgtype {
            let request = client
                .encryption()
                .get_verification_request(&event.sender, &event.event_id)
                .await
                .expect("Request object wasn't created");

            tokio::spawn(request_verification_handler(request));
        }
    });

    client.sync(SyncSettings::default()).await?;

    Ok(())
}

async fn request_verification_handler(request: VerificationRequest) {
    info!(target: LIB_LOG_TARGET, "Accepting verification request from {}", request.other_user_id(),);
    request.accept().await.expect("Can't accept verification request");

    let mut stream = request.changes();

    while let Some(state) = stream.next().await {
        match state {
            VerificationRequestState::Created { .. }
            | VerificationRequestState::Requested { .. }
            | VerificationRequestState::Ready { .. } => (),
            VerificationRequestState::Transitioned { verification } => {
                if let Verification::SasV1(s) = verification {
                    tokio::spawn(sas_verification_handler(s));
                    break;
                }
            }
            VerificationRequestState::Done | VerificationRequestState::Cancelled(_) => break,
        }
    }
}

async fn sas_verification_handler(sas: SasVerification) {
    info!(target: LIB_LOG_TARGET, "Starting verification with {} {}", &sas.other_device().user_id(), &sas.other_device().device_id());
    sas.accept().await.unwrap();

    let mut stream = sas.changes();

    while let Some(state) = stream.next().await {
        match state {
            SasState::KeysExchanged { emojis, decimals: _ } => {
                tokio::spawn(wait_for_confirmation(
                    sas.clone(),
                    emojis.expect("We only support verifications using emojis").emojis,
                ));
            }
            SasState::Done { .. } => {
                let device = sas.other_device();

                info!(target: LIB_LOG_TARGET,
                    "Successfully verified device {} {} {:?}",
                    device.user_id(),
                    device.device_id(),
                    device.local_trust_state()
                );

                break;
            }
            SasState::Cancelled(cancel_info) => {
                warn!(target: LIB_LOG_TARGET, "The verification has been cancelled, reason: {}", cancel_info.reason());

                break;
            }
            SasState::Started { .. } | SasState::Accepted { .. } | SasState::Confirmed => (),
        }
    }
}

async fn wait_for_confirmation(sas: SasVerification, emoji: [Emoji; 7]) {
    info!(target: LIB_LOG_TARGET, "\nDo the emojis match: \n{}", format_emojis(emoji));
    print!("Confirm with `yes` or cancel with `no`: ");
    std::io::stdout().flush().expect("We should be able to flush stdout");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("error: unable to read user input");

    match input.trim().to_lowercase().as_ref() {
        "yes" | "true" | "y" => sas.confirm().await.unwrap(),
        _ => sas.cancel().await.unwrap(),
    }
}
