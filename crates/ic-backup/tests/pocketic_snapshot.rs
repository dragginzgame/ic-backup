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
