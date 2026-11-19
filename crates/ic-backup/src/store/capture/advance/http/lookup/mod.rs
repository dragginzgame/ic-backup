//! One externally reserved read-state dispatch through the existing HTTP owner.

use super::{BackupHttpError, BackupHttpTransport, remaining};
use ic_platform::ingress::decode_read_state_response;

impl BackupHttpTransport {
    // The owning operation admits the exact signed lookup and durable reservation.
    pub(in crate::store) async fn read_state_certificate(
        &self,
        canister: ic_platform_types::Principal,
        lookup: &[u8],
        request_id: [u8; 32],
        deadline: u64,
        maximum: usize,
    ) -> Result<Vec<u8>, BackupHttpError> {
        let mut url = self.origin.clone();
        url.set_path(&format!(
            "/api/v2/canister/{}/read_state",
            canister.to_text()
        ));
        let bytes = self
            .post_cbor(url, lookup.to_vec(), deadline, maximum)
            .await?;
        let certificate = decode_read_state_response(&bytes, maximum)
            .map_err(|_| BackupHttpError::Response)?
            .value
            .certificate;
        remaining(deadline)
            .map_err(|error| error.after_response(request_id, certificate.clone()))?;

        Ok(certificate)
    }
}
