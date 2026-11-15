use super::super::*;
use crate::store::{
    CaptureFinalStatusError, StatusProgress,
    capture::{
        advance::tests::{
            fixtures::certificates::{reply_tree, signed},
            fixtures::{
                data::Fixture,
                http::{HttpProvider, Server, runtime},
                metadata::response,
                signatures::{Scheme, change},
            },
        },
        status::tests::status,
    },
};
use std::{fs, time::Duration};

pub(in crate::store) fn assembled(network: u32) -> Fixture {
    let mut f = Fixture::with_limits(false, false, network, 16 * 1024 * 1024);
    for n in 0..2 {
        f.advance(n).unwrap();
    }
    runtime()
        .block_on(f.op.assemble_data(&f.t, &mut f.provider))
        .unwrap();
    f
}

pub(in crate::store) fn request(f: &Fixture, reuse: bool) -> PreparedStatus {
    let mut r = PreparedStatus::decode(
        &fs::read(f.ledger().parent().unwrap().join("status-request.bin")).unwrap(),
    )
    .unwrap();
    if !reuse {
        change(&mut r, |e| {
            if let EnvelopeContent::Call { nonce, .. } = e.content.to_mut() {
                *nonce = Some(vec![98]);
            }
            e.sender_sig = Some(Scheme::Ed25519.sign(&e.content.to_request_id().signable()));
        });
    }
    r
}

pub(in crate::store) fn result(f: &Fixture, r: &PreparedStatus, running: bool) -> StatusEvidence {
    let mut status = status(&f.p);
    if running {
        status.status = ic_platform::management_canister_types::CanisterStatusType::Running;
    }
    let response = candid_codec::encode_one(&status, 4096).unwrap();
    StatusEvidence {
        proof: signed(reply_tree(&r.request_id, &response, b"replied"), 1, None),
        response,
        rejected: false,
    }
}

pub(in crate::store) fn provider(
    f: &Fixture,
    r: PreparedStatus,
    e: &StatusEvidence,
) -> (Server, HttpProvider) {
    let server = Server::new(response(e));
    let p = HttpProvider {
        transport: server.client(&f.p, Duration::from_secs(1)),
        prepared: Some(r),
        replay: false,
    };
    (server, p)
}

pub(in crate::store) fn advance(
    f: &mut Fixture,
    a: &StatusAdmissionRecord,
    p: &mut HttpProvider,
) -> Result<StatusProgress, CaptureFinalStatusError<StatusHttpError>> {
    runtime().block_on(f.op.advance_final_status(a, &f.t, p))
}
