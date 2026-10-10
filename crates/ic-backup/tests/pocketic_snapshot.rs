//! Real `PocketIC` management journeys through the public extracted implementation.

#![cfg(unix)]

mod pic_journey;
mod support;

#[test]
fn planned_download_uses_one_original_stage_and_publishes_complete_artifact() {
    pic_journey::planned_download::run(pic_journey::planned_download::ReadFailure::None);
}
#[test]
fn planned_download_lost_reply_retains_pending_stage_and_partial_bytes() {
    pic_journey::planned_download::run(pic_journey::planned_download::ReadFailure::Lost);
}
#[test]
fn planned_download_malformed_reply_retains_pending_stage_without_followup() {
    pic_journey::planned_download::run(pic_journey::planned_download::ReadFailure::Malformed);
}

#[test]
fn planned_capture_lost_reply_retains_pending_original_without_recapture_or_successor() {
    pic_journey::planned_download::capture_failure(
        pic_journey::planned_download::CaptureFailure::Lost,
    );
}

#[test]
fn planned_capture_malformed_reply_retains_pending_original_without_recapture_or_successor() {
    pic_journey::planned_download::capture_failure(
        pic_journey::planned_download::CaptureFailure::Malformed,
    );
}

#[test]
fn real_capture_download_upload_and_same_id_restore_preserve_complete_state() {
    pic_journey::run(pic_journey::Fault::None);
}

#[test]
fn lost_capture_reply_requires_reserved_reconciliation_and_never_reissues() {
    pic_journey::run(pic_journey::Fault::Capture);
}

#[test]
fn lost_metadata_reply_reconciles_original_allocation_before_data_upload() {
    pic_journey::run(pic_journey::Fault::Metadata);
}

#[test]
fn lost_data_reply_reconciles_original_extent_without_reupload() {
    pic_journey::run(pic_journey::Fault::Data);
}

#[test]
fn lost_stop_reply_reconciles_original_ingress_without_restop() {
    pic_journey::run(pic_journey::Fault::Stop);
}

#[test]
fn lost_load_reply_requires_complete_stopped_state_verification_before_start() {
    pic_journey::run(pic_journey::Fault::Load);
}

#[test]
fn lost_start_reply_reconciles_original_ingress_without_restart() {
    pic_journey::run(pic_journey::Fault::Start);
}

#[test]
fn lost_load_and_status_replies_keep_pending_reservations_and_stop_before_restart() {
    pic_journey::run(pic_journey::Fault::LoadObservation);
}

#[test]
fn lost_metadata_read_reply_keeps_original_spending_and_stops_before_download() {
    pic_journey::lost_transfer_read(false);
}

#[test]
fn lost_data_read_reply_retains_created_artifacts_without_reissue_or_upload() {
    pic_journey::lost_transfer_read(true);
}

#[test]
fn planned_metadata_lost_reply_retains_pending_without_data_stage_or_reissue() {
    pic_journey::planned_download::metadata_failure(
        pic_journey::planned_download::ReadFailure::Lost,
    );
}

#[test]
fn planned_metadata_malformed_reply_retains_bytes_without_data_stage_or_reissue() {
    pic_journey::planned_download::metadata_failure(
        pic_journey::planned_download::ReadFailure::Malformed,
    );
}

#[test]
fn planned_upload_uses_original_allocation_and_exact_data_stages() {
    pic_journey::planned_upload::run(pic_journey::planned_upload::Failure::None);
}
#[test]
fn planned_upload_lost_metadata_stops_without_data_stage_or_reissue() {
    pic_journey::planned_upload::run(pic_journey::planned_upload::Failure::MetadataLost);
}
#[test]
fn planned_upload_malformed_metadata_stops_without_data_stage_or_reissue() {
    pic_journey::planned_upload::run(pic_journey::planned_upload::Failure::MetadataMalformed);
}
#[test]
fn planned_upload_lost_data_retains_source_and_pending_without_reissue() {
    pic_journey::planned_upload::run(pic_journey::planned_upload::Failure::DataLost);
}
#[test]
fn planned_upload_malformed_data_retains_reply_source_and_pending_without_reissue() {
    pic_journey::planned_upload::run(pic_journey::planned_upload::Failure::DataMalformed);
}

#[test]
fn planned_upload_lost_second_data_write_retains_applied_and_pending_without_reissue() {
    pic_journey::planned_upload::run(pic_journey::planned_upload::Failure::DataLostSecond);
}
#[test]
fn planned_upload_malformed_second_data_write_retains_original_history_without_reissue() {
    pic_journey::planned_upload::run(pic_journey::planned_upload::Failure::DataMalformedSecond);
}
#[test]
fn planned_upload_insufficient_original_data_allowance_refuses_before_writes() {
    pic_journey::planned_upload::run(pic_journey::planned_upload::Failure::DataAllowance);
}
