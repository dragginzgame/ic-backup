//! Real `PocketIC` management journeys through the public extracted implementation.

#![cfg(unix)]

mod pic_journey;
mod ready;
mod support;

#[test]
fn original_workflow_runner_coordinates_real_capture_and_local_replay() {
    pic_journey::planned_runner::run(false);
}

#[test]
fn original_workflow_runner_lost_capture_refuses_redispatch() {
    pic_journey::planned_runner::run(true);
}

#[test]
fn original_workflow_runner_completes_capture_metadata_and_data_artifact() {
    pic_journey::planned_runner::run_complete(pic_journey::planned_download::ReadFailure::None);
}

#[test]
fn original_workflow_runner_lost_data_retains_prefix_and_refuses_redispatch() {
    pic_journey::planned_runner::run_complete(pic_journey::planned_download::ReadFailure::Lost);
}

#[test]
fn original_workflow_runner_replays_retained_prefix_before_data_stage() {
    pic_journey::planned_runner::run_retained();
}

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
fn planned_download_lost_first_reply_retains_created_bytes_without_followup() {
    pic_journey::planned_download::run(pic_journey::planned_download::ReadFailure::LostFirst);
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

#[test]
fn planned_restore_lost_load_retains_pending_without_start_or_reissue() {
    pic_journey::planned_upload::run(pic_journey::planned_upload::Failure::LoadLost);
}
#[test]
fn planned_restore_malformed_load_retains_reply_without_start_or_reissue() {
    pic_journey::planned_upload::run(pic_journey::planned_upload::Failure::LoadMalformed);
}
#[test]
fn planned_restore_lost_start_retains_pending_without_restart() {
    pic_journey::planned_upload::run(pic_journey::planned_upload::Failure::StartLost);
}
#[test]
fn planned_restore_malformed_start_retains_reply_without_restart() {
    pic_journey::planned_upload::run(pic_journey::planned_upload::Failure::StartMalformed);
}

#[test]
fn connected_original_workflow_round_trips_same_id_and_replays_without_effects() {
    pic_journey::planned_runner::run_round_trip(pic_journey::planned_upload::Failure::None, false);
}

#[test]
fn connected_original_workflow_resumes_at_allocation_load_and_start() {
    pic_journey::planned_runner::run_round_trip(pic_journey::planned_upload::Failure::None, true);
}

#[test]
fn connected_original_workflow_lost_allocation_refuses_reentry_and_upload() {
    pic_journey::planned_runner::run_round_trip(
        pic_journey::planned_upload::Failure::MetadataLost,
        false,
    );
}

#[test]
fn connected_original_workflow_lost_second_upload_retains_applied_prefix_and_pending() {
    pic_journey::planned_runner::run_round_trip(
        pic_journey::planned_upload::Failure::DataLostSecond,
        false,
    );
}

#[test]
fn connected_original_workflow_lost_load_refuses_reentry_and_start() {
    pic_journey::planned_runner::run_round_trip(
        pic_journey::planned_upload::Failure::LoadLost,
        false,
    );
}

#[test]
fn connected_original_workflow_lost_start_retains_pending_without_reissue() {
    pic_journey::planned_runner::run_round_trip(
        pic_journey::planned_upload::Failure::StartLost,
        false,
    );
}
