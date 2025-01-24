use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiNaDevices {
    pub data: Vec<MiNADevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiNADevice {
    pub address: String,
    pub alias: String,
    #[serde(rename = "brokerEndpoint")]
    pub broker_endpoint: String,
    #[serde(rename = "brokerIndex")]
    pub broker_index: i32,
    pub current: bool,
    #[serde(rename = "deviceID")]
    pub id: String,
    #[serde(rename = "deviceProfile")]
    pub profile: String,
    #[serde(rename = "deviceSNProfile")]
    pub sn_profile: String,
    pub hardware: String,
    pub mac: String,
    #[serde(rename = "miotDID")]
    pub miot_did: String,
    pub name: String,
    pub presence: String,
    #[serde(rename = "remoteCtrlType")]
    pub remote_ctrl_type: String,
    #[serde(rename = "romVersion")]
    pub rom_version: String,
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    pub ssid: String,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    #[serde(rename = "ai_instruction")]
    pub ai_instruction: i32,
    #[serde(rename = "ai_protocol_3_0")]
    pub ai_protocol_3_0: i32,
    #[serde(rename = "alarm_repeat_option_v2")]
    pub alarm_repeat_option_v2: i32,
    #[serde(rename = "alarm_volume")]
    pub alarm_volume: i32,
    #[serde(rename = "baby_schedule")]
    pub baby_schedule: i32,
    #[serde(rename = "bluetooth_option_v2")]
    pub bluetooth_option_v2: i32,
    #[serde(rename = "child_mode")]
    pub child_mode: i32,
    #[serde(rename = "child_mode_2")]
    pub child_mode_2: i32,
    #[serde(rename = "classified_alarm")]
    pub classified_alarm: i32,
    #[serde(rename = "content_blacklist")]
    pub content_blacklist: i32,
    #[serde(rename = "cp_level")]
    pub cp_level: i32,
    #[serde(rename = "dialog_h5")]
    pub dialog_h5: i32,
    pub dlna: i32,
    pub earthquake: i32,
    #[serde(rename = "family_voice")]
    pub family_voice: i32,
    #[serde(rename = "lan_tv_control")]
    pub lan_tv_control: i32,
    #[serde(rename = "loadmore_v2")]
    pub loadmore_v2: i32,
    pub mesh: i32,
    #[serde(rename = "mico_current")]
    pub mico_current: i32,
    #[serde(rename = "nearby_wakeup_cloud")]
    pub nearby_wakeup_cloud: i32,
    #[serde(rename = "night_mode")]
    pub night_mode: i32,
    #[serde(rename = "night_mode_detail")]
    pub night_mode_detail: i32,
    #[serde(rename = "night_mode_v2")]
    pub night_mode_v2: i32,
    #[serde(rename = "player_pause_timer")]
    pub player_pause_timer: i32,
    #[serde(rename = "report_times")]
    pub report_times: i32,
    #[serde(rename = "school_timetable")]
    pub school_timetable: i32,
    #[serde(rename = "skill_try")]
    pub skill_try: i32,
    #[serde(rename = "tone_setting")]
    pub tone_setting: i32,
    #[serde(rename = "user_nick_name")]
    pub user_nick_name: i32,
    #[serde(rename = "voice_print")]
    pub voice_print: i32,
    #[serde(rename = "voice_print_multidevice")]
    pub voice_print_multidevice: i32,
    #[serde(rename = "voip_used_time")]
    pub voip_used_time: i32,
    #[serde(rename = "xiaomi_voip")]
    pub xiaomi_voip: i32,
    pub yueyu: i32,
    pub yunduantts: i32,
}
