use super::ASIAirState;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

pub fn get_app_state(_: &Option<Value>, state: Arc<Mutex<ASIAirState>>) -> (Value, u8) {
    let state = state.lock().unwrap();

    (
        json!({
            "page": state.page.as_str(),
            "annotate": {
                "is_working": state.annotate.is_working,
                "lapse_ms": state.annotate.lapse_ms
            },
            "solve": {
                "is_working": state.solve.is_working,
                "lapse_ms": state.solve.lapse_ms,
                "filename": state.solve.filename
            },
            "capture": {
                "exposure_mode": state.capture.exposure_mode.as_str(),
                "is_working": state.capture.is_working,
                "state": state.capture.state.as_str(),
            },
            "pa": {
                "is_working": state.pa.is_working,
            },
            "auto_goto": {
                "is_working": state.auto_goto.is_working,
            },
            "stack": {
                "is_working": state.stack.is_working,
                "frame_type": state.stack.frame_type.as_str(),
                "stacked_frame": state.stack.stacked_frame,
                "dropped_frame": state.stack.dropped_frame,
                "total_frame": state.stack.total_frame
            },
            "export_image": {
                "is_working": state.export_image.is_working,
                "success_frame": state.export_image.success_frame,
                "total_frame": state.export_image.total_frame,
                "keep": state.export_image.keep,
                "dst_storage": state.export_image.dst_storage.as_str(),
            },
            "merid_flip": {
                "is_working": state.merid_flip.is_working,
            },
            "auto_focus": {
                "result": {},
                "is_working": state.auto_focus.is_working,
                "focuser_opened": state.auto_focus.focuser_opened,
                "reason": {
                    "comment": state.auto_focus.reason.comment,
                    "code": state.auto_focus.reason.code,
                }
            },
            "find_star": {
                "is_working": state.find_star.is_working,
                "lapse_ms": state.find_star.lapse_ms,
            },
            "avi_record": {
                "is_working": state.avi_record.is_working,
                "lapse_sec": state.avi_record.lapse_sec,
                "fps": state.avi_record.fps,
                "write_file_fps": state.avi_record.write_file_fps,
            },
            "rtmp": {
                "is_working": state.rtmp.is_working,
            },
            "auto_exp": {
                "is_working": state.auto_exp.is_working,
            },
            "restart_guide": {
                "is_working": state.restart_guide.is_working,
            },
            "batch_stack": {
                "is_working": state.batch_stack.is_working,
            },
            "demonstrate": {
                "is_working": state.demonstrate.is_working,
            },
            "format_drive": {
                "is_working": state.format_drive.is_working,
            },
        }),
        0,
    )
}
