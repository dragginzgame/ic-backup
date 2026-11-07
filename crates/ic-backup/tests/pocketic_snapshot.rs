//! Real `PocketIC` management journeys through the public extracted implementation.

#![cfg(unix)]

mod pic_journey;
mod ready;
#[expect(
    dead_code,
    reason = "this integration target uses only a subset of the shared fixture helpers"
)]
mod support;

#[tokio::test]
async fn original_workflow_runner_coordinates_real_capture_and_local_replay() {
    pic_journey::planned_runner::run(false).await;
}

#[tokio::test]
async fn original_workflow_runner_lost_capture_refuses_redispatch() {
    pic_journey::planned_runner::run(true).await;
}

#[tokio::test]
async fn original_workflow_runner_completes_capture_metadata_and_data_artifact() {
    pic_journey::planned_runner::run_complete(pic_journey::planned_download::ReadFailure::None)
        .await;
}

#[tokio::test]
async fn original_workflow_runner_lost_data_retains_prefix_and_refuses_redispatch() {
    pic_journey::planned_runner::run_complete(pic_journey::planned_download::ReadFailure::Lost(1))
        .await;
}

#[tokio::test]
async fn original_workflow_runner_replays_retained_prefix_before_data_stage() {
    pic_journey::planned_runner::run_retained().await;
}

#[tokio::test]
async fn planned_download_malformed_reply_retains_pending_stage_without_followup() {
    pic_journey::planned_runner::run_complete(
        pic_journey::planned_download::ReadFailure::Malformed(1),
    )
    .await;
}

#[tokio::test]
async fn planned_capture_lost_reply_retains_pending_original_without_recapture_or_successor() {
    pic_journey::planned_download::capture_failure(
        pic_journey::planned_download::CaptureFailure::Lost,
    )
    .await;
}

#[tokio::test]
async fn planned_capture_malformed_reply_retains_pending_original_without_recapture_or_successor() {
    pic_journey::planned_download::capture_failure(
        pic_journey::planned_download::CaptureFailure::Malformed,
    )
    .await;
}

#[tokio::test]
async fn lost_capture_reply_requires_reserved_reconciliation_and_never_reissues() {
    pic_journey::run(pic_journey::Fault::Capture).await;
}

#[tokio::test]
async fn lost_metadata_reply_reconciles_original_allocation_before_data_upload() {
    pic_journey::run(pic_journey::Fault::Metadata).await;
}

#[tokio::test]
async fn lost_data_reply_reconciles_original_extent_without_reupload() {
    pic_journey::run(pic_journey::Fault::Data).await;
}

#[tokio::test]
async fn lost_stop_reply_reconciles_original_ingress_without_restop() {
    pic_journey::run(pic_journey::Fault::Stop).await;
}

#[tokio::test]
async fn lost_load_reply_requires_complete_stopped_state_verification_before_start() {
    pic_journey::run(pic_journey::Fault::Load).await;
}

#[tokio::test]
async fn lost_start_reply_reconciles_original_ingress_without_restart() {
    pic_journey::run(pic_journey::Fault::Start).await;
}

#[tokio::test]
async fn lost_load_and_status_replies_keep_pending_reservations_and_stop_before_restart() {
    pic_journey::run(pic_journey::Fault::LoadObservation).await;
}

#[tokio::test]
async fn planned_download_lost_first_reply_retains_created_bytes_without_followup() {
    pic_journey::planned_runner::run_complete(pic_journey::planned_download::ReadFailure::Lost(0))
        .await;
}

#[tokio::test]
async fn planned_metadata_lost_reply_retains_pending_without_data_stage_or_reissue() {
    pic_journey::planned_download::metadata_failure(
        pic_journey::planned_download::ReadFailure::Lost(0),
    )
    .await;
}

#[tokio::test]
async fn planned_metadata_malformed_reply_retains_bytes_without_data_stage_or_reissue() {
    pic_journey::planned_download::metadata_failure(
        pic_journey::planned_download::ReadFailure::Malformed(0),
    )
    .await;
}
