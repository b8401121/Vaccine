#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct VaccineItem {
    name: String,
    dose_info: String,
    timing_info: String,
    category: String, // "Routine" (公費常規) | "SelfPaid" (自費建議) | "Subsidized" (縣市補助)
    description: String,
    audience: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct TimelineMilestone {
    title: String,
    age_months: i32,
    target_date: String,
    status: String, // "Past", "Current", "Next"
    vaccines: Vec<VaccineItem>,
    co_admin_guide: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct VaccineResponse {
    age_display: String,
    child_age_detail: String,
    gender_display: String,
    location_display: String,
    current_visit_date: String,
    current_visit_milestone: String,
    next_visit_date: String,
    next_visit_milestone: String,
    milestones: Vec<TimelineMilestone>,
}

#[derive(Serialize, Deserialize, Clone)]
struct VaccineDetailDoc {
    id: String,
    name: String,
    aliases: String,
    category: String, // "Routine" | "SelfPaid" | "Both"
    target_audience: String, // "兒童/青少年" | "成人/長者" | "全齡通用"
    prevent_disease: String,
    full_description: String,
    schedule: Vec<String>,
    notes: String,
}

struct MilestoneSpec {
    title: &'static str,
    min_month: i32,
    max_month: i32,
    vaccines: Vec<VaccineItem>,
}

fn get_co_admin_guide(min_month: i32, is_adult: bool, is_travel: bool) -> Vec<String> {
    if is_travel {
        return vec![
            "黃皮書國際疫苗：黃熱病疫苗 (活性) 與腦膜炎雙球菌疫苗 (不活化) 可同天於旅遊門診不同手臂/大腿部位施打。".into(),
            "⚠️ 活性疫苗同次規則：黃熱病疫苗為活性減毒疫苗，若與 MMR 或水痘未在同一天施打，需間隔至少 28 天 (4 週) 以上。".into(),
            "抗瘧疾預防藥物 (Malarone / Doxycycline / Mefloquine) 經口服吸收，與針劑疫苗無相斥性，按醫囑時間服用即可。".into(),
        ];
    }

    if is_adult {
        return vec![
            "💉 成人同次最佳組合：流感疫苗 (每年1劑) 可與 COVID-19 最新株、PCV20 肺炎鏈球菌疫苗或 Tdap 破傷風疫苗，於『同一次就診分左右手臂』完成施打！".into(),
            "✅ 衛福部與 ACIP 指引：所有成人不活化疫苗 (流感、COVID-19、PCV20、Tdap、HPV、Shingrix、A肝) 均可於同次分開左右部位施打，無間隔日數限制。".into(),
        ];
    }

    match min_month {
        0 => vec![
            "💉 施打部位：B型肝炎第1劑 (不活化) 於大腿前外側施打。".into(),
            "💡 高風險提醒：母親若為 B 肝帶原者，新生兒需於 24 小時內於另一大腿部位加打 1 劑 B 肝免疫球蛋白 (HBIG)。".into(),
        ],
        1 => vec![
            "💉 單獨施打 B 型肝炎第2劑 (不活化)。若 delay 可於滿 2 個月時與五合一疫苗於不同大腿部位同次補打。".into(),
        ],
        2 => vec![
            "💉 左大腿：五合一疫苗 第 1 劑 (不活化)".into(),
            "💉 右大腿：13 價肺炎鏈球菌 PCV13 第 1 劑 (不活化)".into(),
            "🍼 口服：輪狀病毒疫苗 第 1 劑 (口服減毒，於針劑施打前後口服)".into(),
            "💡 自費腸病毒 EV71：可同次施打於另一大腿部位 (與同腿針劑間隔至少 2.5cm)，或擇日施打。".into(),
        ],
        4 => vec![
            "💉 左大腿：五合一疫苗 第 2 劑 (不活化)".into(),
            "💉 右大腿：13 價肺炎鏈球菌 PCV13 第 2 劑 (不活化)".into(),
            "🍼 口服：輪狀病毒疫苗 第 2 劑 (口服給藥)".into(),
            "💡 自費腸病毒 EV71：可同次施打或擇日完成。".into(),
        ],
        5 => vec![
            "💉 卡介苗 (BCG) 為活性減毒皮內注射疫苗。".into(),
            "⚠️ 活性疫苗叮嚀：若與其他活性減毒針劑 (如水痘/MMR) 未於同天施打，必須間隔至少 28 天 (4 週) 以上。".into(),
        ],
        6 => vec![
            "💉 左大腿：五合一疫苗 第 3 劑 (不活化)".into(),
            "💉 右大腿：B 型肝炎疫苗 第 3 劑 (不活化)".into(),
            "🍂 秋冬季節：滿6個月以上可同次追加『季節性流感疫苗』於另一部位。".into(),
            "🍼 口服：輪狀病毒疫苗第 3 劑 (若選用 3 劑型廠牌)。".into(),
        ],
        12 => vec![
            "⚠️ 活性減毒疫苗核心間隔金律：水痘疫苗與 MMR 疫苗均為活性減毒疫苗！".into(),
            "💉 方案 A (最佳同次完成)：同一次門診完成：左手臂/大腿『MMR 第1劑』 + 右手臂/大腿『水痘 第1劑』 + 大腿『PCV13 追加劑』。".into(),
            "⏰ 方案 B (若不同天打)：若『水痘』與『MMR』未在同一天施打，則兩者必須【間隔至少 28 天 (4 週)】以上方可施打另一支！".into(),
            "💡 自費 A 肝疫苗：為不活化疫苗，可於同次施打於不同部位或間隔任意時間。".into(),
        ],
        15 => vec![
            "💉 左手臂/大腿：日本腦炎疫苗 第 1 劑 (活性減毒)".into(),
            "💡 活性疫苗叮嚀：活性日本腦炎疫苗可與 A 肝疫苗 (不活化) 同次施打；若與滿1歲水痘/MMR 未同天打，需間隔至少 28 天。".into(),
        ],
        18 => vec![
            "💉 左大腿/手臂：五合一疫苗 第 4 劑追加劑 (不活化)".into(),
            "💉 右大腿/手臂：A 型肝炎疫苗 第 1 劑 (公費不活化)".into(),
            "✅ 不活化疫苗組合：可完全於同一次就診分開左右肢體接種完成！".into(),
        ],
        27 => vec![
            "💉 左手臂：日本腦炎疫苗 第 2 劑 (活性減毒，與第1劑隔12個月)".into(),
            "💉 右手臂：A 型肝炎疫苗 第 2 劑 (不活化，與第1劑隔6個月)".into(),
            "✅ 可同次施打於左右手臂或大腿部位。".into(),
        ],
        48 => vec![
            "💡 自費追加水痘第 2 劑：小兒感染症醫學會建議於 4-6 歲自費追加第 2 劑。若與 MMR 第 2 劑同時打可分左右臂；不同天打需間隔至少 28 天。".into(),
        ],
        60 => vec![
            "💉 左上臂三角肌：四合一疫苗 DTaP-IPV 追加劑 (不活化)".into(),
            "💉 右上臂三角肌：MMR 混合疫苗 第 2 劑 (活性減毒)".into(),
            "✅ 國小入學前兩大重要疫苗，建議於幼兒園大班升國小暑假同次於左右上臂完成施打！".into(),
        ],
        _ => vec![
            "💉 凡不活化疫苗均可於同一次就診在不同部位同時接種；活性減毒疫苗若不同天打需間隔 28 天以上。".into(),
        ],
    }
}

#[cfg(target_os = "windows")]
fn ensure_webview2_loader() {
    let loader_bytes = include_bytes!("../WebView2Loader.dll");
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let dll_path = exe_dir.join("WebView2Loader.dll");
            if !dll_path.exists() {
                let _ = std::fs::write(&dll_path, loader_bytes);
            }
        }
    }
}

#[tauri::command]
fn get_eligible_vaccines(
    year: i32,
    month: u32,
    day: u32,
    is_roc: bool,
    gender: String,
    location: String,
) -> Result<VaccineResponse, String> {
    let actual_year = if is_roc { year + 1911 } else { year };
    
    let dob = NaiveDate::from_ymd_opt(actual_year, month, day)
        .ok_or("無效的日期")?;
        
    let now = Local::now().naive_local().date();
    if dob > now {
        return Err("出生日期不能在未來".into());
    }

    let mut age_years = now.year() - dob.year();
    let mut age_months = now.month() as i32 - dob.month() as i32;
    let age_days = now.day() as i32 - dob.day() as i32;

    if age_days < 0 {
        age_months -= 1;
    }
    if age_months < 0 {
        age_years -= 1;
        age_months += 12;
    }

    let total_months = (age_years * 12) + age_months;

    let age_display = if age_years > 0 {
        if age_months > 0 {
            format!("{} 歲 {} 個月", age_years, age_months)
        } else {
            format!("{} 歲", age_years)
        }
    } else {
        format!("{} 個月", age_months.max(0))
    };

    let child_age_detail = if total_months <= 216 {
        if age_years > 0 {
            format!("{} 歲 {} 個月 (相當於 {} 個月大)", age_years, age_months, total_months)
        } else {
            format!("{} 個月大", age_months.max(0))
        }
    } else {
        format!("{} 歲", age_years)
    };

    let is_female = gender == "female";
    let gender_display = if is_female { "女性 ♀" } else { "男性 ♂" }.to_string();
    let location_display = if location.is_empty() { "全國標準".into() } else { location.clone() };

    let is_taipei = location.contains("台北") || location.contains("臺北");
    let is_new_taipei = location.contains("新北");
    let is_taoyuan = location.contains("桃園");
    let is_taichung = location.contains("台中") || location.contains("臺中");
    let is_tainan = location.contains("台南") || location.contains("臺南");
    let is_kaohsiung = location.contains("高雄");
    let is_island = location.contains("金門") || location.contains("連江") || location.contains("澎湖");

    // 輪狀病毒縣市補助描述
    let rotavirus_desc = if is_taipei {
        "🏛️ 台北市獨家定額補助：設籍台北市一般幼兒定額補助 2,100 元！低收/中低收/罕病/第3胎以上全額公費免費。".to_string()
    } else if is_new_taipei || is_taoyuan || is_taichung || is_tainan || is_kaohsiung {
        format!("🏛️ {}弱勢補助：低收、中低收入戶及特定山地原住民區幼兒享全額免費補助，一般幼兒自費口服。", location)
    } else if is_island {
        format!("🏛️ {}離島補助：離島地區提供弱勢及特定幼兒全額免費補助，一般幼兒自費口服。", location)
    } else {
        "自費口服疫苗 (2劑型或3劑型)。註：多數縣市針對低收/中低收/第3胎以上幼兒提供全額公費補助。".to_string()
    };

    let rotavirus_cat = if is_taipei { "Subsidized" } else { "SelfPaid" };

    let mut milestones_out = Vec::new();

    let mut child_specs = vec![
        MilestoneSpec {
            title: "出生 24 小時內",
            min_month: 0,
            max_month: 1,
            vaccines: vec![VaccineItem {
                name: "B 型肝炎疫苗".into(),
                dose_info: "第 1 劑".into(),
                timing_info: "出生 24 小時內".into(),
                category: "Routine".into(),
                description: "新生兒出生後儘速施打公費第 1 劑".into(),
                audience: "Children".into(),
            }],
        },
        MilestoneSpec {
            title: "出生滿 1 個月",
            min_month: 1,
            max_month: 2,
            vaccines: vec![VaccineItem {
                name: "B 型肝炎疫苗".into(),
                dose_info: "第 2 劑".into(),
                timing_info: "滿 1 個月".into(),
                category: "Routine".into(),
                description: "公費基礎劑第 2 劑".into(),
                audience: "Children".into(),
            }],
        },
        MilestoneSpec {
            title: "出生滿 2 個月",
            min_month: 2,
            max_month: 4,
            vaccines: vec![
                VaccineItem {
                    name: "五合一疫苗 (DTaP-Hib-IPV)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 2 個月".into(),
                    category: "Routine".into(),
                    description: "公費預防白喉、破傷風、百日咳、B型嗜血桿菌、小兒麻痺".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "13 價結合型肺炎鏈球菌疫苗 (PCV13)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 2 個月".into(),
                    category: "Routine".into(),
                    description: "公費基礎劑第 1 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "輪狀病毒疫苗 (Rotavirus)".into(),
                    dose_info: if is_taipei { "北市補助 / 自費第 1 劑" } else { "自費口服第 1 劑" }.into(),
                    timing_info: "滿 2 個月".into(),
                    category: rotavirus_cat.into(),
                    description: rotavirus_desc.clone(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "腸病毒 A71 型疫苗 (EV71)".into(),
                    dose_info: "自費第 1 劑".into(),
                    timing_info: "滿 2 個月至 6 歲".into(),
                    category: "SelfPaid".into(),
                    description: "自費接種，預防腸病毒 A71 型併發重症".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "出生滿 4 個月",
            min_month: 4,
            max_month: 5,
            vaccines: vec![
                VaccineItem {
                    name: "五合一疫苗 (DTaP-Hib-IPV)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 4 個月".into(),
                    category: "Routine".into(),
                    description: "公費基礎劑第 2 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "13 價結合型肺炎鏈球菌疫苗 (PCV13)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 4 個月".into(),
                    category: "Routine".into(),
                    description: "公費基礎劑第 2 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "輪狀病毒疫苗 (Rotavirus)".into(),
                    dose_info: if is_taipei { "北市補助 / 自費第 2 劑" } else { "自費口服第 2 劑" }.into(),
                    timing_info: "滿 4 個月".into(),
                    category: rotavirus_cat.into(),
                    description: rotavirus_desc.clone(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "腸病毒 A71 型疫苗 (EV71)".into(),
                    dose_info: "自費第 2 劑".into(),
                    timing_info: "滿 4 個月 (與第1劑隔2月)".into(),
                    category: "SelfPaid".into(),
                    description: "自費基礎劑第 2 劑".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "出生滿 5-8 個月",
            min_month: 5,
            max_month: 6,
            vaccines: vec![VaccineItem {
                name: "卡介苗 (BCG)".into(),
                dose_info: "單劑".into(),
                timing_info: "滿 5-8 個月".into(),
                category: "Routine".into(),
                description: "公費建議於滿 5-8 個月施打".into(),
                audience: "Children".into(),
            }],
        },
        MilestoneSpec {
            title: "出生滿 6 個月",
            min_month: 6,
            max_month: 12,
            vaccines: vec![
                VaccineItem {
                    name: "五合一疫苗 (DTaP-Hib-IPV)".into(),
                    dose_info: "第 3 劑".into(),
                    timing_info: "滿 6 個月".into(),
                    category: "Routine".into(),
                    description: "公費基礎劑第 3 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "B 型肝炎疫苗".into(),
                    dose_info: "第 3 劑".into(),
                    timing_info: "滿 6 個月".into(),
                    category: "Routine".into(),
                    description: "公費基礎劑第 3 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "季節性流感疫苗".into(),
                    dose_info: "第 1-2 劑 (初次接種隔4週)".into(),
                    timing_info: "滿 6 個月以上".into(),
                    category: "Routine".into(),
                    description: "滿6個月即可接種流感疫苗，8歲以下初次接種需打2劑".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "出生滿 12 個月",
            min_month: 12,
            max_month: 15,
            vaccines: vec![
                VaccineItem {
                    name: "水痘疫苗 (Varicella)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 12 個月".into(),
                    category: "Routine".into(),
                    description: "公費滿 12 個月施打第 1 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "麻疹腮腺炎德國麻疹混合疫苗 (MMR)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 12 個月".into(),
                    category: "Routine".into(),
                    description: "公費滿 12 個月施打第 1 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "13 價結合型肺炎鏈球菌疫苗 (PCV13)".into(),
                    dose_info: "第 3 劑 (追加劑)".into(),
                    timing_info: "滿 12-15 個月".into(),
                    category: "Routine".into(),
                    description: "公費滿 12-15 個月施打追加劑".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "出生滿 15 個月",
            min_month: 15,
            max_month: 18,
            vaccines: vec![VaccineItem {
                name: "日本腦炎疫苗 (JE)".into(),
                dose_info: "第 1 劑".into(),
                timing_info: "滿 15 個月".into(),
                category: "Routine".into(),
                description: "公費滿 15 個月施打第 1 劑".into(),
                audience: "Children".into(),
            }],
        },
        MilestoneSpec {
            title: "出生滿 18 個月",
            min_month: 18,
            max_month: 27,
            vaccines: vec![
                VaccineItem {
                    name: "五合一疫苗 (DTaP-Hib-IPV)".into(),
                    dose_info: "第 4 劑 (追加劑)".into(),
                    timing_info: "滿 18 個月".into(),
                    category: "Routine".into(),
                    description: "公費滿 18 個月追加第 4 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "A 型肝炎疫苗 (Hep A)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 18 個月".into(),
                    category: "Routine".into(),
                    description: "公費滿 18 個月施打第 1 劑".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "出生滿 27 個月",
            min_month: 27,
            max_month: 60,
            vaccines: vec![
                VaccineItem {
                    name: "日本腦炎疫苗 (JE)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 27 個月".into(),
                    category: "Routine".into(),
                    description: "公費與第 1 劑隔至少 12 個月".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "A 型肝炎疫苗 (Hep A)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 27 個月".into(),
                    category: "Routine".into(),
                    description: "公費與第 1 劑隔至少 6 個月".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "滿 4-6 歲幼兒追加自費",
            min_month: 48,
            max_month: 60,
            vaccines: vec![VaccineItem {
                name: "水痘疫苗 (Varicella)".into(),
                dose_info: "自費第 2 劑 (追加劑)".into(),
                timing_info: "滿 4-6 歲".into(),
                category: "SelfPaid".into(),
                description: "小兒感染症醫學會建議於 4-6 歲自費追加第 2 劑水痘疫苗，保護力更佳".into(),
                audience: "Children".into(),
            }],
        },
        MilestoneSpec {
            title: "滿 5 歲至國小入學前",
            min_month: 60,
            max_month: 108,
            vaccines: vec![
                VaccineItem {
                    name: "麻疹腮腺炎德國麻疹混合疫苗 (MMR)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 5 歲至國小入學前".into(),
                    category: "Routine".into(),
                    description: "公費入學前完成施打第 2 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "白喉破傷風百日咳及小兒麻痺疫苗 (DTaP-IPV)".into(),
                    dose_info: "追加劑 (第 5 劑)".into(),
                    timing_info: "滿 5 歲至國小入學前".into(),
                    category: "Routine".into(),
                    description: "公費入學前追加 1 劑".into(),
                    audience: "Children".into(),
                },
            ],
        },
    ];

    // 男國中生 HPV 縣市政策判斷
    let hpv_boy_subsidized = is_taipei || is_new_taipei || is_taoyuan;

    let hpv_adolescent_routine = if is_female {
        vec![
            VaccineItem {
                name: "人類乳突病毒疫苗 (HPV)".into(),
                dose_info: "公費 2 劑 (按 0, 6 個月)".into(),
                timing_info: "國中女生 (約12-15歲)".into(),
                category: "Routine".into(),
                description: "衛福部國健署提供國中女生公費施打 2 劑 9 價 HPV 疫苗，有效預防子宮頸癌及 HPV 相關癌症".into(),
                audience: "Children".into(),
            },
            VaccineItem {
                name: "人類乳突病毒疫苗 (HPV)".into(),
                dose_info: "自費 2 劑 (按 0, 6 個月)".into(),
                timing_info: "9-14 歲青少年".into(),
                category: "SelfPaid".into(),
                description: "非公費補助資格之 9-14 歲女生，可選擇自費接種 2 劑 9 價 HPV 疫苗".into(),
                audience: "Children".into(),
            },
        ]
    } else {
        if hpv_boy_subsidized {
            vec![
                VaccineItem {
                    name: "人類乳突病毒疫苗 (HPV)".into(),
                    dose_info: format!("{}公費 2 劑 (按 0, 6 個月)", location),
                    timing_info: "國中男生 (約12-15歲)".into(),
                    category: "Subsidized".into(),
                    description: format!("🏛️ {}地方政府擴大福利：提供國中男學生與女學生同享免費公費施打 9 價 HPV 疫苗！", location),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "人類乳突病毒疫苗 (HPV)".into(),
                    dose_info: "自費 2 劑 (按 0, 6 個月)".into(),
                    timing_info: "9-14 歲男性青少年".into(),
                    category: "SelfPaid".into(),
                    description: "非公費補助學校之 9-14 歲男性，建議自費接種 2 劑 4價或9價 HPV 疫苗。".into(),
                    audience: "Children".into(),
                },
            ]
        } else {
            vec![VaccineItem {
                name: "人類乳突病毒疫苗 (HPV)".into(),
                dose_info: "自費 2 劑 (按 0, 6 個月)".into(),
                timing_info: "9-14 歲男性青少年".into(),
                category: "SelfPaid".into(),
                description: "9-14 歲男性建議自費接種 2 劑 4價或9價 HPV 疫苗，預防尖形濕疣(菜花)及陰莖癌/肛門癌等".into(),
                audience: "Children".into(),
            }]
        }
    };

    child_specs.push(MilestoneSpec {
        title: "滿 9 歲至 14 歲 (青少年/國中階段)",
        min_month: 108,
        max_month: 180,
        vaccines: hpv_adolescent_routine,
    });

    let hpv_highschool = vec![VaccineItem {
        name: "人類乳突病毒疫苗 (HPV)".into(),
        dose_info: "自費 3 劑 (按 0, 2, 6 個月)".into(),
        timing_info: "15 歲以上青少年".into(),
        category: "SelfPaid".into(),
        description: if is_female {
            "15歲以上首次接種需施打 3 劑 (按 0-2-6 個月時程)，防範 HPV 病毒感染及子宮頸癌"
        } else {
            "15歲以上男性首次接種需施打 3 劑 (按 0-2-6 個月時程)，防範 HPV 病毒感染及口咽癌/菜花"
        }.into(),
        audience: "Children".into(),
    }];

    child_specs.push(MilestoneSpec {
        title: "滿 15 歲至 18 歲 (高中/青年階段)",
        min_month: 180,
        max_month: 216,
        vaccines: hpv_highschool,
    });

    if total_months <= 216 {
        for spec in child_specs {
            let status = if total_months >= spec.max_month {
                "Past"
            } else if total_months >= spec.min_month && total_months < spec.max_month {
                "Current"
            } else {
                "Next"
            };

            let add_years = (spec.min_month / 12) as i32;
            let add_months = (spec.min_month % 12) as u32;
            let mut target_year = dob.year() + add_years;
            let mut target_month = dob.month() + add_months;
            if target_month > 12 {
                target_year += ((target_month - 1) / 12) as i32;
                target_month = ((target_month - 1) % 12) + 1;
            }
            let target_day = dob.day().min(28);
            let target_date_str = format!("{:04}-{:02}-{:02}", target_year, target_month, target_day);

            let co_guide = get_co_admin_guide(spec.min_month, false, false);
            milestones_out.push(TimelineMilestone {
                title: spec.title.to_string(),
                age_months: spec.min_month,
                target_date: target_date_str,
                status: status.to_string(),
                vaccines: spec.vaccines,
                co_admin_guide: co_guide,
            });
        }
    }

    if age_years >= 18 {
        let mut adult_routine = vec![
            VaccineItem {
                name: "季節性流感疫苗".into(),
                dose_info: "每年 1 劑".into(),
                timing_info: "秋冬流感季".into(),
                category: "Routine".into(),
                description: "所有成人每年建議施打 1 劑 (公費或自費選用優質疫苗)".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "新冠疫苗 (COVID-19)".into(),
                dose_info: "依政策 1-2 劑".into(),
                timing_info: "定期追加".into(),
                category: "Routine".into(),
                description: "提升對抗主流病毒株之免疫保護力".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "減量破傷風、白喉、百日咳疫苗 (Tdap)".into(),
                dose_info: "自費第 1 劑 / 每10年追加".into(),
                timing_info: "成人常規".into(),
                category: "SelfPaid".into(),
                description: if is_female {
                    "成人建議每10年自費追加1劑。女性每次懷孕(27-36週)均建議施打1劑以傳遞抗體給嬰兒。"
                } else {
                    "成人建議每10年自費追加1劑，預防破傷風、白喉與百日咳。"
                }.into(),
                audience: "Adults".into(),
            },
        ];

        if age_years >= 19 && age_years <= 45 {
            let hpv_desc = if is_female {
                "自費HPV疫苗：2價僅適用女性，4價與9價適用男女性。可預防子宮頸癌及相關病變。(按 0-2-6 個月時程施打 3 劑)"
            } else {
                "自費HPV疫苗：4價與9價適用於男性。可預防尖形濕疣(菜花)及肛門癌等病變。(按 0-2-6 個月時程施打 3 劑)"
            };

            adult_routine.push(VaccineItem {
                name: "人類乳突病毒疫苗 (HPV)".into(),
                dose_info: "自費共 3 劑 (0-2-6 個月)".into(),
                timing_info: if age_years <= 26 { "19-26 歲建議" } else { "27-45 歲自費" }.into(),
                category: "SelfPaid".into(),
                description: hpv_desc.into(),
                audience: "Adults".into(),
            });
        }

        if age_years >= 18 {
            let shingles_desc = if is_taoyuan || is_island {
                format!("🏛️ {}福利：65歲以上低收/中低收入長者享地方政府補助！一般人自費 2 劑 (隔 2-6 月)。", location)
            } else {
                "預防帶狀疱疹(皮蛇)及疱疹後神經痛，防護率達90%以上。按 0, 2-6 個月施打 2 劑 (50歲以上或18歲以上高風險)。".to_string()
            };

            adult_routine.push(VaccineItem {
                name: "非活性帶狀疱疹疫苗 (Shingrix)".into(),
                dose_info: "自費共 2 劑 (隔 2-6 月)".into(),
                timing_info: "50 歲以上或 18 歲以上高風險".into(),
                category: if is_taoyuan || is_island { "Subsidized".into() } else { "SelfPaid".into() },
                description: shingles_desc,
                audience: "Adults".into(),
            });
        }

        if age_years >= 65 || (age_years >= 55 && age_years <= 64) {
            adult_routine.push(VaccineItem {
                name: "肺炎鏈球菌疫苗 (PCV13/PCV20/PPV23)".into(),
                dose_info: "公費/自費 1-2 劑".into(),
                timing_info: if age_years >= 65 { "65 歲以上" } else { "55-64 歲原住民" }.into(),
                category: "Routine".into(),
                description: "公費提供 1 劑 PCV13/PCV20 銜接 PPV23。未達公費年齡者可自費接種".into(),
                audience: "Adults".into(),
            });
        }

        if age_years >= 60 {
            let rsv_desc = if is_female {
                "60歲以上長者建議自費1劑。懷孕婦女(28-36週)接種可傳遞被動免疫給嬰兒。"
            } else {
                "60歲以上長者建議自費1劑，預防RSV引發之下呼吸道疾病與重症。"
            };

            adult_routine.push(VaccineItem {
                name: "呼吸道細胞融合病毒 (RSV) 疫苗".into(),
                dose_info: "自費 1 劑".into(),
                timing_info: "60 歲以上長者".into(),
                category: "SelfPaid".into(),
                description: rsv_desc.into(),
                audience: "Adults".into(),
            });
        }

        let adult_co_guide = get_co_admin_guide(total_months, true, false);
        milestones_out.push(TimelineMilestone {
            title: format!("成人常規與自費建議疫苗 ({}, {}, {})", age_display, gender_display, location_display),
            age_months: total_months,
            target_date: now.format("%Y-%m-%d").to_string(),
            status: "Current".to_string(),
            vaccines: adult_routine,
            co_admin_guide: adult_co_guide,
        });

        let travel_specs = vec![
            VaccineItem {
                name: "A 型肝炎疫苗".into(),
                dose_info: "自費共 2 劑 (隔 6-12 個月)".into(),
                timing_info: "高風險/赴流行區".into(),
                category: "SelfPaid".into(),
                description: "慢性肝病或頻繁赴東南亞/大陸等流行地區者建議自費接種".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "流行性腦脊髓膜炎疫苗 (MenACWY / MenB)".into(),
                dose_info: "自費 1-2 劑".into(),
                timing_info: "赴中東/留學/高風險區".into(),
                category: "SelfPaid".into(),
                description: "沙烏地朝聖或赴歐美留學特定學校強制要求接種之自費疫苗".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "黃熱病疫苗 (Yellow Fever)".into(),
                dose_info: "自費 1 劑 (終生有效)".into(),
                timing_info: "赴非洲/南美洲前10天".into(),
                category: "SelfPaid".into(),
                description: "旅遊醫學門診專用自費疫苗，入境特定國家所需之國際預防接種證明(黃皮書)".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "傷寒疫苗 (Typhoid Vaccine)".into(),
                dose_info: "自費 1 劑 (效期 3 年)".into(),
                timing_info: "赴流行區前 2 週".into(),
                category: "SelfPaid".into(),
                description: "旅遊醫學門診自費疫苗，前往高風險衛生不良地區旅遊前接種".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "狂犬病疫苗 (Rabies Vaccine)".into(),
                dose_info: "自費 3 劑 (按 0, 7, 21-28 天)".into(),
                timing_info: "暴露前預防/高風險工作".into(),
                category: "SelfPaid".into(),
                description: "獸醫、野生動物研究人員或前往狂犬病高風險國家野生動物接觸者".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "麻疹腮腺炎德國麻疹疫苗 (MMR)".into(),
                dose_info: if is_female && age_years >= 15 && age_years <= 49 { "育齡公費 1 劑 / 自費 1-2 劑" } else { "自費 1-2 劑 (隔 28 天)" }.into(),
                timing_info: "育齡婦女/出國旅遊".into(),
                category: if is_female && age_years >= 15 && age_years <= 49 { "Routine".into() } else { "SelfPaid".into() },
                description: if is_female && age_years >= 15 && age_years <= 49 {
                    "15-49歲育齡婦女檢具近3個月德國麻疹抗體陰性證明者可公費接種1劑。其他人自費補打。"
                } else {
                    "1966年後出生或不具抗體者出國前建議自費補打 1-2 劑。"
                }.into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "M 痘疫苗 (Mpox Vaccine)".into(),
                dose_info: "共 2 劑 (間隔 4 週)".into(),
                timing_info: "具風險行為者".into(),
                category: "SelfPaid".into(),
                description: "暴露前/後預防與風險行為者自費/公費接種".into(),
                audience: "Adults".into(),
            },
        ];

        let travel_co_guide = get_co_admin_guide(total_months + 1, false, true);
        milestones_out.push(TimelineMilestone {
            title: "旅遊醫學與特定自費疫苗 (CDC 清單)".to_string(),
            age_months: total_months + 1,
            target_date: now.format("%Y-%m-%d").to_string(),
            status: "Current".to_string(),
            vaccines: travel_specs,
            co_admin_guide: travel_co_guide,
        });
    }

    let mut current_visit_date = String::new();
    let mut current_visit_milestone = String::new();
    let mut next_visit_date = String::new();
    let mut next_visit_milestone = String::new();

    let today_str = now.format("%Y-%m-%d").to_string();

    for m in &milestones_out {
        if m.status == "Current" && current_visit_date.is_empty() {
            // 當次 → 一律使用程式執行當天日期
            current_visit_date = today_str.clone();
            current_visit_milestone = m.title.clone();
        } else if m.status == "Next" && next_visit_date.is_empty() {
            next_visit_date = m.target_date.clone();
            next_visit_milestone = m.title.clone();
        }
    }

    if current_visit_date.is_empty() {
        current_visit_date = today_str.clone();
        current_visit_milestone = "當前階段 (即日起符合常規施打)".into();
    }
    if next_visit_date.is_empty() {
        next_visit_date = "定期常規追蹤 / 每年秋冬流感季".into();
        next_visit_milestone = "年度定期保養 / 每年秋冬".into();
    }

    Ok(VaccineResponse {
        age_display,
        child_age_detail,
        gender_display,
        location_display,
        current_visit_date,
        current_visit_milestone,
        next_visit_date,
        next_visit_milestone,
        milestones: milestones_out,
    })
}

#[tauri::command]
fn get_all_vaccines() -> Vec<VaccineDetailDoc> {
    vec![
        VaccineDetailDoc {
            id: "dtap-hib-ipv".into(),
            name: "五合一疫苗 (DTaP-Hib-IPV)".into(),
            aliases: "白喉破傷風非細胞性百日咳、b型嗜血桿菌及不活化小兒麻痺混合疫苗".into(),
            category: "Routine".into(),
            target_audience: "兒童/幼兒".into(),
            prevent_disease: "白喉、破傷風、百日咳、b型嗜血桿菌侵襲性感染、小兒麻痺症".into(),
            full_description: "五合一疫苗為幼兒基礎免疫之核心疫苗，取代舊型全細胞百日咳疫苗，大幅降低發燒等不良反應。".into(),
            schedule: vec![
                "第 1 劑：出生滿 2 個月 (公費)".into(),
                "第 2 劑：出生滿 4 個月 (公費)".into(),
                "第 3 劑：出生滿 6 個月 (公費)".into(),
                "第 4 劑 (追加劑)：出生滿 18 個月 (公費)".into(),
            ],
            notes: "若接種後出現高燒不退或嚴重過敏反應，下一劑應由醫師評估。".into(),
        },
        VaccineDetailDoc {
            id: "hepb".into(),
            name: "B 型肝炎疫苗 (Hepatitis B)".into(),
            aliases: "B肝疫苗".into(),
            category: "Both".into(),
            target_audience: "全齡通用".into(),
            prevent_disease: "B 型肝炎病毒感染、慢性肝炎、肝硬化與肝癌".into(),
            full_description: "台灣自1984年推行新生兒B肝疫苗接種，顯著降低幼兒帶原率。成人檢驗無抗體者亦建議自費補打。".into(),
            schedule: vec![
                "第 1 劑：出生 24 小時內儘速接種 (公費)".into(),
                "第 2 劑：出生滿 1 個月 (公費)".into(),
                "第 3 劑：出生滿 6 個月 (公費)".into(),
                "成人補打：按 0, 1, 6 個月時程自費接種 3 劑".into(),
            ],
            notes: "高風險孕婦所生之新生兒，需於出生24小時內另外追加注射1劑B型肝炎免疫球蛋白(HBIG)。".into(),
        },
        VaccineDetailDoc {
            id: "bcg".into(),
            name: "卡介苗 (BCG)".into(),
            aliases: "結核病疫苗".into(),
            category: "Routine".into(),
            target_audience: "嬰幼兒".into(),
            prevent_disease: "結核性腦膜炎及結核病播散性感染".into(),
            full_description: "用於預防幼兒嚴重結核病。接種後局部會有小紅結節與膿疱，為正常免疫反應過程。".into(),
            schedule: vec![
                "單劑：建議於出生滿 5 至 8 個月時施打 (公費)".into(),
            ],
            notes: "若嬰幼兒有免疫缺損或疑似免疫缺陷家族史，嚴禁接種卡介苗。".into(),
        },
        VaccineDetailDoc {
            id: "pcv".into(),
            name: "結合型肺炎鏈球菌疫苗 (PCV13 / PCV20)".into(),
            aliases: "13價/20價肺炎鏈球菌疫苗".into(),
            category: "Both".into(),
            target_audience: "全齡通用 (幼兒與65歲以上長者)".into(),
            prevent_disease: "肺炎鏈球菌引發之肺炎、敗血症及腦膜炎性侵襲性疾病".into(),
            full_description: "提供強效抗原結合免疫記憶。公費提供幼兒3劑PCV13，以及65歲以上長者1劑PCV13/20銜接PPV23。".into(),
            schedule: vec![
                "幼兒第 1 劑：出生滿 2 個月 (公費)".into(),
                "幼兒第 2 劑：出生滿 4 個月 (公費)".into(),
                "幼兒第 3 劑 (追加)：出生滿 12-15 個月 (公費)".into(),
                "65歲以上長者：公費提供 1 劑 PCV13 / PCV20，間隔1年銜接 1 劑 PPV23".into(),
                "一般成人自費：建議高風險或50歲以上成人自費接種 1 劑 PCV20".into(),
            ],
            notes: "未滿 65 歲之高風險慢性病患者，建議諮詢醫師提前自費接種。".into(),
        },
        VaccineDetailDoc {
            id: "varicella".into(),
            name: "水痘疫苗 (Varicella)".into(),
            aliases: "水痘水泡疫苗".into(),
            category: "Both".into(),
            target_audience: "幼兒與無抗體青少年/成人".into(),
            prevent_disease: "水痘病毒感染、全身性水疱及繼發性細菌感染".into(),
            full_description: "活性減毒疫苗。公費補助1歲幼兒打第1劑，4-6歲小兒科醫學會強烈建議自費追加第2劑以達到完美保護力。".into(),
            schedule: vec![
                "幼兒第 1 劑：出生滿 12 個月 (公費)".into(),
                "幼兒第 2 劑：滿 4-6 歲 (建議自費追加)".into(),
                "13歲以上未曾感染且無抗體者：自費接種 2 劑 (間隔 4-8 週)".into(),
            ],
            notes: "孕婦及嚴重免疫抑制者禁忌接種活性減毒水痘疫苗。".into(),
        },
        VaccineDetailDoc {
            id: "mmr".into(),
            name: "麻疹腮腺炎德國麻疹混合疫苗 (MMR)".into(),
            aliases: "三合一MMR疫苗".into(),
            category: "Both".into(),
            target_audience: "全齡通用 (幼兒公費 / 育齡婦女公費 / 出國自費)".into(),
            prevent_disease: "麻疹、流行性腮腺炎、德國麻疹及先天性德國麻疹症候群 (CRS)".into(),
            full_description: "極高保護效力之活性減毒疫苗。育齡婦女若無德國麻疹抗體可公費接種。前往日本/東南亞前建議評估補打。".into(),
            schedule: vec![
                "第 1 劑：出生滿 12 個月 (公費)".into(),
                "第 2 劑：滿 5 歲至國小入學前 (公費)".into(),
                "15-49歲育齡婦女：檢具德國麻疹抗體陰性證明可公費接種 1 劑".into(),
                "1966年後出生/出國人員：可自費補打 1-2 劑 (間隔 28 天以上)".into(),
            ],
            notes: "接種後4週內應避免懷孕。孕婦禁忌接種。".into(),
        },
        VaccineDetailDoc {
            id: "je".into(),
            name: "日本腦炎疫苗 (JE)".into(),
            aliases: "日腦疫苗 (細胞培養活性減毒疫苗)".into(),
            category: "Both".into(),
            target_audience: "幼兒與成人高風險/旅遊".into(),
            prevent_disease: "日本腦炎病毒引發之急性腦炎與神經性後遺症".into(),
            full_description: "現行公費採用新型細胞培養活性減毒疫苗，僅需接種 2 劑即可產生長期免疫。".into(),
            schedule: vec![
                "第 1 劑：出生滿 15 個月 (公費)".into(),
                "第 2 劑：出生滿 27 個月 (公費，與第1劑隔12個月)".into(),
                "成人自費：經常於豬舍、水稻田周邊工作或旅遊者可自費補打 1 劑".into(),
            ],
            notes: "若有發燒或急性感染應延期接種。".into(),
        },
        VaccineDetailDoc {
            id: "hepa".into(),
            name: "A 型肝炎疫苗 (Hepatitis A)".into(),
            aliases: "A肝疫苗".into(),
            category: "Both".into(),
            target_audience: "幼兒公費 / 成人自費".into(),
            prevent_disease: "A 型肝炎病毒引起的急性肝炎、黃疸與急性肝衰竭".into(),
            full_description: "幼兒全面納入公費施打。成人慢性肝病患者、餐飲從業人員或前往流行地區者強烈建議自費接種。".into(),
            schedule: vec![
                "幼兒第 1 劑：出生滿 18 個月 (公費)".into(),
                "幼兒第 2 劑：出生滿 27 個月 (公費，隔6-12個月)".into(),
                "成人自費：按 0, 6-12 個月時程施打 2 劑 (自費)".into(),
            ],
            notes: "完整施打 2 劑後，免疫力可維持 20 年以上。".into(),
        },
        VaccineDetailDoc {
            id: "dtap-ipv".into(),
            name: "白喉破傷風百日咳及小兒麻痺疫苗 (DTaP-IPV 追加劑)".into(),
            aliases: "四合一疫苗 (入學前追加劑)".into(),
            category: "Routine".into(),
            target_audience: "滿 5 歲至國小入學前兒童".into(),
            prevent_disease: "白喉、破傷風、百日咳及小兒麻痺症".into(),
            full_description: "作為幼兒小學前之強化追加劑，維繫百日咳與小兒麻痺之社群抗體屏障。".into(),
            schedule: vec![
                "單劑 (第5劑)：滿 5 歲至國小入學前完成施打 (公費)".into(),
            ],
            notes: "國小入學查驗項目之一，建議於開學前完成。".into(),
        },
        VaccineDetailDoc {
            id: "rotavirus".into(),
            name: "輪狀病毒疫苗 (Rotavirus Vaccine)".into(),
            aliases: "輪狀口服疫苗".into(),
            category: "SelfPaid".into(),
            target_audience: "嬰幼兒 (2-8個月大)".into(),
            prevent_disease: "輪狀病毒引發之嬰幼兒嚴重嘔吐、水瀉、脫水與住院".into(),
            full_description: "口服活性減毒疫苗。北市定額補助$2,100元，低收/中低收/罕病/第3胎全額公費免費。".into(),
            schedule: vec![
                "2劑型：出生滿 2 個月、4 個月各口服 1 劑".into(),
                "3劑型：出生滿 2 個月、4 個月、6 個月各口服 1 劑".into(),
            ],
            notes: "嬰兒若有腸套疊病史或腸道先天畸形者禁忌口服。".into(),
        },
        VaccineDetailDoc {
            id: "ev71".into(),
            name: "腸病毒 A71 型疫苗 (EV71 Vaccine)".into(),
            aliases: "腸病毒疫苗".into(),
            category: "SelfPaid".into(),
            target_audience: "滿2個月至6歲幼兒".into(),
            prevent_disease: "腸病毒 A71 型引發之手足口病、腦炎、肺水腫等重症與死亡".into(),
            full_description: "台灣國產研發非活性疫苗 (如高端、國光/安特羅)，專門針對最易引發重症的 A71 型病毒株。".into(),
            schedule: vec![
                "基礎劑第 1 劑：滿 2 個月至 6 歲以下 (自費)".into(),
                "基礎劑第 2 劑：與第 1 劑間隔 2 個月 (自費)".into(),
                "追加劑：依廠牌規定於 1 歲後追加 1 劑 (自費)".into(),
            ],
            notes: "本疫苗僅針對 A71 型，對其他型別腸病毒 (如克沙奇病毒) 無直接交叉保護。".into(),
        },
        VaccineDetailDoc {
            id: "hpv".into(),
            name: "人類乳突病毒疫苗 (HPV Vaccine)".into(),
            aliases: "子宮頸癌疫苗 / 九價 HPV 疫苗".into(),
            category: "Both".into(),
            target_audience: "國中女生(公費) / 北市.新北.桃園男國中生(公費) / 9-45歲(自費)".into(),
            prevent_disease: "子宮頸癌、外陰癌、陰道癌、菜花 (尖形濕疣)、肛門癌及口咽癌".into(),
            full_description: "具備極高預防效果。國健署提供國中女生公費9價疫苗；台北市、新北市、桃園市特別擴大提供國中男學生公費接種！".into(),
            schedule: vec![
                "國中女生/特定縣市男國中生公費：2 劑 (按 0, 6 個月時程)".into(),
                "9-14歲男女自費：2 劑 (按 0, 6 個月時程)".into(),
                "15-45歲男女自費：3 劑 (按 0, 2, 6 個月時程)".into(),
            ],
            notes: "發燒或懷孕期間請延期接種。已發生性行為者接種仍具保護力。".into(),
        },
        VaccineDetailDoc {
            id: "shingles".into(),
            name: "非活性帶狀疱疹疫苗 (Shingrix)".into(),
            aliases: "皮蛇疫苗 / 欣安立適".into(),
            category: "SelfPaid".into(),
            target_audience: "50歲以上成人 / 18歲以上高風險對象".into(),
            prevent_disease: "帶狀疱疹 (皮蛇) 及長期疱疹後神經痛 (PHN)".into(),
            full_description: "基因重組非活性疫苗，保護力長達10年以上，防護率達90%-97%。桃園、嘉義市、離島針對高齡弱勢長者享地方補助。".into(),
            schedule: vec![
                "第 1 劑：50歲以上或18歲以上免疫低下者 (自費/地方補助)".into(),
                "第 2 劑：與第 1 劑間隔 2 至 6 個月 (自費/地方補助)".into(),
            ],
            notes: "無論過去是否罹患過皮蛇，均建議接種。".into(),
        },
        VaccineDetailDoc {
            id: "flu".into(),
            name: "季節性流感疫苗 (Influenza Vaccine)".into(),
            aliases: "流感疫苗 (三價/四價/高劑量/細胞培養)".into(),
            category: "Both".into(),
            target_audience: "全齡通用 (6個月以上)".into(),
            prevent_disease: "季節性 A 型與 B 型流感及其引發之肺炎重症與死亡".into(),
            full_description: "每年秋冬開打。公費提供幼兒、長者、孕婦、慢性病患；非公費對象亦建議每年自費施打1劑。".into(),
            schedule: vec![
                "6個月以上至8歲初次接種：施打 2 劑 (間隔 4 週)".into(),
                "9歲以上及成人：每年接種 1 劑 (公費或自費)".into(),
            ],
            notes: "流感病毒變異快，必須每年重新接種最新病毒株之疫苗。".into(),
        },
        VaccineDetailDoc {
            id: "covid19".into(),
            name: "新冠病毒疫苗 (COVID-19 Vaccine)".into(),
            aliases: "COVID-19 疫苗 (JN.1 / 最新株)".into(),
            category: "Routine".into(),
            target_audience: "6個月大以上所有民眾".into(),
            prevent_disease: "新型冠狀病毒感染引發之重症、住院與死亡".into(),
            full_description: "依衛生福利部最新政策定期開打，建議每年秋冬與流感疫苗同時或間隔接種追加劑。".into(),
            schedule: vec![
                "滿6個月以上民眾：按最新衛福部規範接種 1-2 劑公費疫苗".into(),
            ],
            notes: "可與流感疫苗或肺炎鏈球菌疫苗同時在不同部位接種。".into(),
        },
        VaccineDetailDoc {
            id: "tdap".into(),
            name: "減量破傷風白喉百日咳疫苗 (Tdap)".into(),
            aliases: "Tdap 三合一疫苗 (成人型)".into(),
            category: "SelfPaid".into(),
            target_audience: "成人、孕婦及新手父母小兒照顧者".into(),
            prevent_disease: "成人與新生兒之百日咳、破傷風及白喉".into(),
            full_description: "成人百日咳抗體會隨時間衰退。建議所有成人每10年追加1劑。孕婦每胎次27-36週自費接種可保護新生兒。".into(),
            schedule: vec![
                "成人常規：建議每 10 年自費追加 1 劑".into(),
                "孕婦：每次懷孕第 27-36 週自費接種 1 劑".into(),
            ],
            notes: "產婦若產前未接種，應於生產後儘速補打 1 劑。".into(),
        },
        VaccineDetailDoc {
            id: "rsv".into(),
            name: "呼吸道細胞融合病毒疫苗 (RSV Vaccine)".into(),
            aliases: "RSV 疫苗 (Arexvy / Abrysvo)".into(),
            category: "SelfPaid".into(),
            target_audience: "60歲以上長者 / 孕婦(28-36週)".into(),
            prevent_disease: "RSV 引起的嚴重下呼吸道感染、肺炎、氣喘發作與住院".into(),
            full_description: "最新核准之蛋白重組疫苗。長者接種可防範重症；孕婦於懷孕後期接種可將抗體傳遞給嬰兒防護6個月。".into(),
            schedule: vec![
                "60歲以上長者：自費施打 1 劑".into(),
                "懷孕婦女：懷孕第 28-36 週自費施打 1 劑".into(),
            ],
            notes: "提供長者及無法直接打疫苗之新生兒絕佳被動免疫保護。".into(),
        },
        VaccineDetailDoc {
            id: "yellow-fever".into(),
            name: "黃熱病疫苗 (Yellow Fever Vaccine)".into(),
            aliases: "黃皮書疫苗 / 旅遊醫學門診".into(),
            category: "SelfPaid".into(),
            target_audience: "前往非洲、中南美洲流行地區旅客".into(),
            prevent_disease: "黃熱病病毒引發之急性黃疸、出血與肝腎衰竭".into(),
            full_description: "活性減毒疫苗。前往特定國家強制要求出示國際預防接種證明書 (黃皮書)，需於出國前10天於旅遊醫學門診施打。".into(),
            schedule: vec![
                "出國前 10 天於指定旅遊醫學門診自費施打 1 劑 (終生有效)".into(),
            ],
            notes: "蛋類嚴重過敏者、免疫缺損者或9個月以下嬰兒禁忌接種。".into(),
        },
        VaccineDetailDoc {
            id: "meningococcal".into(),
            name: "流行性腦脊髓膜炎疫苗 (Meningococcal Vaccine)".into(),
            aliases: "腦膜炎雙球菌疫苗 (MenACWY / MenB)".into(),
            category: "SelfPaid".into(),
            target_audience: "赴歐美留學生 / 沙烏地朝聖者 / 高風險工作".into(),
            prevent_disease: "腦膜炎雙球菌引發之劇烈頭痛、腦膜炎及爆發性敗血症".into(),
            full_description: "涵蓋 A, C, W, Y 型或 B 型。歐美多所大學強制入學前施打；赴麥加朝聖者亦為入境簽證強制規定。".into(),
            schedule: vec![
                "MenACWY (結合型)：自費接種 1 劑 (保護力約 5 年)".into(),
                "MenB (B型專用)：自費接種 2 劑 (間隔 1 個月)".into(),
            ],
            notes: "前往高風險宿舍環境前建議提前 2 週完成接種。".into(),
        },
        VaccineDetailDoc {
            id: "typhoid".into(),
            name: "傷寒疫苗 (Typhoid Vaccine)".into(),
            aliases: "傷寒多醣體疫苗".into(),
            category: "SelfPaid".into(),
            target_audience: "前往高風險衛生不良地區旅客".into(),
            prevent_disease: "傷寒桿菌引發之持續高燒、腹痛、腸出血與脾臟腫大".into(),
            full_description: "前往南亞 (如印度、尼泊爾)、東南亞衛生條件較差地區前建議至旅遊門診自費施打。".into(),
            schedule: vec![
                "出國前 2 週自費施打 1 劑 (保護效期為 3 年，持續高風險每3年追加)".into(),
            ],
            notes: "滿 2 歲以上方可接種本疫苗。".into(),
        },
        VaccineDetailDoc {
            id: "rabies".into(),
            name: "狂犬病疫苗 (Rabies Vaccine)".into(),
            aliases: "狂犬病不活化疫苗".into(),
            category: "SelfPaid".into(),
            target_audience: "暴露前預防 (獸醫/動物研究/旅遊) 或 暴露後處置".into(),
            prevent_disease: "狂犬病病毒引發之致命性急性腦脊髓炎 (致死率近100%)".into(),
            full_description: "不活化疫苗。前往狂犬病流行國野外活動或動物高風險接觸者建議預先完成 3 劑暴露前預防。".into(),
            schedule: vec![
                "暴露前預防 (自費)：按 0, 7, 21-28 天施打 3 劑".into(),
                "暴露後處置：遭疑似野生動物咬傷後，依醫囑於 0, 3, 7, 14 天補打 4 劑 (公費或自費)".into(),
            ],
            notes: "若遭咬傷應立即以肥皂及大量清水沖洗傷口15分鐘並儘速就醫。".into(),
        },
        VaccineDetailDoc {
            id: "mpox".into(),
            name: "M 痘疫苗 (Mpox Vaccine)".into(),
            aliases: "猴痘疫苗 (JYNNEOS)".into(),
            category: "Both".into(),
            target_audience: "高風險性行為對象 / 照顧者 / 暴露後處置".into(),
            prevent_disease: "M 痘病毒引發之發燒、淋巴腺腫大與全身皮膚水疱病變".into(),
            full_description: "採用第三代非複製型減毒痘病毒疫苗 (JYNNEOS)，安全性極高，副作用輕微。".into(),
            schedule: vec![
                "第 1 劑：符合衛福部條件者公費或高風險自費接種".into(),
                "第 2 劑：與第 1 劑間隔至少 4 週 (28天) 完成完整保護".into(),
            ],
            notes: "皮內注射或皮下注射均可，建議完成 2 劑以獲得最大防護力。".into(),
        },
        VaccineDetailDoc {
            id: "polio-booster".into(),
            name: "小兒麻痺疫苗 (Polio IPV 追加劑與黃皮書)".into(),
            aliases: "IPV 不活化小兒麻痺疫苗追加劑".into(),
            category: "SelfPaid".into(),
            target_audience: "赴阿富汗、巴基斯坦等小兒麻痺流行國旅客".into(),
            prevent_disease: "脊髓灰質炎 (小兒麻痺症) 引起的急性肢體麻痺".into(),
            full_description: "依據 CDC 與 WHO 國際衛生條例規定：前往小兒麻痺野生株或衍生株流行國停留 4 週以上者，出境前 4 週至 12 個月內需追加 1 劑 IPV 並記錄於國際預防接種證明書 (黃皮書)，否則可能被拒絕登機出境！".into(),
            schedule: vec![
                "出境前 4 週至 12 個月內於旅遊門診自費追加 1 劑 IPV (登載黃皮書)".into(),
            ],
            notes: "出國前請務必確認目標國家最新疫情警告與黃皮書規定。".into(),
        },
        VaccineDetailDoc {
            id: "altitude-sickness".into(),
            name: "高山症預防用藥 (Acetazolamide / 丹木斯)".into(),
            aliases: "丹木斯 / 高海拔急性高山病預防用藥".into(),
            category: "SelfPaid".into(),
            target_audience: "赴海拔 >2,500 公尺高山山區旅客".into(),
            prevent_disease: "急性高山病 (AMS)、高海拔腦水腫及高海拔肺水腫".into(),
            full_description: "前往秘魯庫斯科 (Cusco)、印加古道、西藏、衣索比亞高山或台灣高山 (玉山/雪山) 海拔 2,500 公尺以上地區者，建議於旅遊門診評估開立處方用藥。".into(),
            schedule: vec![
                "Acetazolamide (丹木斯)：登高前 24 小時開始服用，每日 2 次，每次 125mg，至適應高海拔後停藥".into(),
            ],
            notes: "磺胺類藥物過敏者禁忌使用；可改用預備用藥。".into(),
        },
        VaccineDetailDoc {
            id: "malaria-med".into(),
            name: "瘧疾預防處方藥品 (Malaria Prophylaxis)".into(),
            aliases: "莫可樂 Malarone / 多喜黴素 Doxycycline / 美爾奎寧 Mefloquine".into(),
            category: "SelfPaid".into(),
            target_audience: "前往非洲、中南美洲、南亞瘧疾流行區旅客".into(),
            prevent_disease: "惡性瘧原蟲引發之高燒、寒顫、溶血性貧血與腦型瘧疾致死重症".into(),
            full_description: "衛福部疾管署核准三大抗瘧疾處方用藥，需於旅遊門診經醫師評估開立，依指示於出發前、行程中及離開後按時服用。".into(),
            schedule: vec![
                "Malarone (莫可樂)：出發前1天開始，每日1顆；離開疫區後繼續吃7天".into(),
                "Doxycycline (多喜黴素)：出發前1-2天開始，每日1顆；離開疫區後繼續吃4週".into(),
                "Mefloquine (美爾奎寧)：出發前2-3週開始，每週1顆；離開疫區後繼續吃4週".into(),
            ],
            notes: "請務必依醫囑按時完成全程服用，切勿自行中途停藥。".into(),
        },
    ]
}

#[derive(Serialize, Deserialize, Clone)]
struct CatchUpResponse {
    vaccine_name: String,
    next_dose_info: String,
    earliest_date_display: String,
    days_remaining: i64,
    is_ready_now: bool,
    acip_rule_summary: String,
    clinical_notes: Vec<String>,
}

#[tauri::command]
fn calculate_catch_up(
    vaccine_id: String,
    last_dose_num: i32,
    year: i32,
    month: u32,
    day: u32,
    is_roc: bool,
) -> Result<CatchUpResponse, String> {
    let actual_year = if is_roc { year + 1911 } else { year };
    let last_date = NaiveDate::from_ymd_opt(actual_year, month, day)
        .ok_or("無效的施打日期")?;
        
    let today = Local::now().naive_local().date();
    if last_date > today {
        return Err("上一劑施打日期不能在未來".into());
    }

    let mut vaccine_name = String::new();
    let mut next_dose_info = String::new();
    let mut min_days_interval: i64 = 28;
    let mut acip_rule_summary = String::new();
    let mut clinical_notes = Vec::new();

    match vaccine_id.as_str() {
        "dtap-hib-ipv" => {
            vaccine_name = "五合一疫苗 (DTaP-Hib-IPV)".into();
            if last_dose_num == 1 {
                next_dose_info = "第 2 劑 (基礎劑)".into();
                min_days_interval = 28;
                acip_rule_summary = "衛福部 ACIP 規範：第 1 劑與第 2 劑至少需間隔 4 週 (28天)。".into();
            } else if last_dose_num == 2 {
                next_dose_info = "第 3 劑 (基礎劑)".into();
                min_days_interval = 28;
                acip_rule_summary = "衛福部 ACIP 規範：第 2 劑與第 3 劑至少需間隔 4 週 (28天)。".into();
            } else {
                next_dose_info = "第 4 劑 (追加劑)".into();
                min_days_interval = 180;
                acip_rule_summary = "衛福部 ACIP 規範：第 4 劑追加劑需與第 3 劑至少間隔 6 個月 (180天)。".into();
                clinical_notes.push("第 4 劑通常建議於出生滿 18 個月施打。".into());
            }
        }
        "hepb" => {
            vaccine_name = "B 型肝炎疫苗 (HepB)".into();
            if last_dose_num == 1 {
                next_dose_info = "第 2 劑".into();
                min_days_interval = 28;
                acip_rule_summary = "衛福部 ACIP 規範：第 1 劑與第 2 劑至少需間隔 4 週 (28天)。".into();
            } else {
                next_dose_info = "第 3 劑".into();
                min_days_interval = 56; // 8 週
                acip_rule_summary = "衛福部 ACIP 規範：第 2 劑與第 3 劑至少需間隔 8 週 (56天)，且距第1劑需至少間隔 16 週。".into();
                clinical_notes.push("若媽媽為 B 肝表面抗原 (HBsAg) 陽性，寶寶出生後已包含 HBIG 免疫球蛋白保護。".into());
            }
        }
        "pcv" => {
            vaccine_name = "13價結合型肺炎鏈球菌疫苗 (PCV13)".into();
            if last_dose_num == 1 {
                next_dose_info = "第 2 劑 (基礎劑)".into();
                min_days_interval = 28;
                acip_rule_summary = "衛福部 ACIP 規範：第 1 劑與第 2 劑至少需間隔 4 週 (28天)。".into();
            } else {
                next_dose_info = "第 3 劑 (追加劑)".into();
                min_days_interval = 56; // 8 週
                acip_rule_summary = "衛福部 ACIP 規範：追加劑需與前一劑至少間隔 8 週 (56天)，且幼兒需滿 12 個月大。".into();
                clinical_notes.push("公費第 3 劑為滿 12-15 個月施打之重要追加劑。".into());
            }
        }
        "rotavirus" => {
            vaccine_name = "輪狀病毒疫苗 (Rotavirus)".into();
            next_dose_info = "下一劑 (自費/地方補助口服)".into();
            min_days_interval = 28;
            acip_rule_summary = "衛福部 ACIP 規範：口服輪狀病毒疫苗各劑之間至少需間隔 4 週 (28天)。".into();
            clinical_notes.push("⚠️ 關鍵年齡上限：第 1 劑最晚需於滿 14 週 6 天前完成。".into());
            clinical_notes.push("⚠️ 關鍵年齡上限：最後 1 劑最晚不可超過滿 8 個月大 (32 週)！若超過年齡上限則不可再補打。".into());
        }
        "varicella" => {
            vaccine_name = "水痘疫苗 (Varicella)".into();
            next_dose_info = "第 2 劑 (追加劑 / 補打)".into();
            min_days_interval = 28;
            acip_rule_summary = "衛福部 ACIP 規範：水痘疫苗第 1 劑與第 2 劑至少需間隔 4 週 (28天)；13歲以上未曾感染者間隔 4-8 週。".into();
            clinical_notes.push("⚠️ 活性減毒疫苗提醒：水痘為活性疫苗。若未在同一天與其他活性疫苗 (如 MMR、日本腦炎) 施打，必須間隔至少 28 天！".into());
        }
        "mmr" => {
            vaccine_name = "麻疹腮腺炎德國麻疹混合疫苗 (MMR)".into();
            next_dose_info = "第 2 劑".into();
            min_days_interval = 28;
            acip_rule_summary = "衛福部 ACIP 規範：MMR 疫苗第 1 劑與第 2 劑至少需間隔 4 週 (28天)。".into();
            clinical_notes.push("⚠️ 活性減毒疫苗提醒：若未與水痘或日本腦炎活性疫苗同一天施打，必須間隔至少 28 天以上。".into());
        }
        "je" => {
            vaccine_name = "日本腦炎疫苗 (JE - 細胞培養減毒)".into();
            next_dose_info = "第 2 劑 (公費)".into();
            min_days_interval = 365; // 12 個月
            acip_rule_summary = "衛福部 ACIP 規範：新型細胞培養活性減毒疫苗第 1 劑與第 2 劑需間隔至少 12 個月 (365天)。".into();
            clinical_notes.push("公費常規於滿 15 個月打第 1 劑，滿 27 個月打第 2 劑。".into());
        }
        "hepa" => {
            vaccine_name = "A 型肝炎疫苗 (HepA)".into();
            next_dose_info = "第 2 劑".into();
            min_days_interval = 180; // 6 個月
            acip_rule_summary = "衛福部 ACIP 規範：A 型肝炎疫苗第 1 劑與第 2 劑需間隔至少 6 個月 (180天)。".into();
            clinical_notes.push("完整施打 2 劑後，免疫保護力可維持 20 年以上。".into());
        }
        "ev71" => {
            vaccine_name = "腸病毒 A71 型疫苗 (EV71)".into();
            if last_dose_num == 1 {
                next_dose_info = "第 2 劑 (基礎劑)".into();
                min_days_interval = 56; // 8 週
                acip_rule_summary = "疫苗廠牌說明：腸病毒 A71 型基礎劑第 1 劑與第 2 劑需間隔 56 天 (8週)。".into();
            } else {
                next_dose_info = "追加劑".into();
                min_days_interval = 365;
                acip_rule_summary = "疫苗廠牌說明：追加劑建議與第 2 劑間隔 12 個月。".into();
            }
            clinical_notes.push("本疫苗適用滿 2 個月至 6 歲以下幼兒自費接種。".into());
        }
        "hpv" => {
            vaccine_name = "九價人類乳突病毒疫苗 (HPV 9價)".into();
            if last_dose_num == 1 {
                next_dose_info = "第 2 劑".into();
                min_days_interval = 150; // 5 個月
                acip_rule_summary = "衛福部 ACIP 規範：9-14歲打2劑(按0, 6月)；15歲以上打3劑(按0, 2, 6月)。第1, 2劑最少需隔 5 個月(150天)。".into();
            } else {
                next_dose_info = "第 3 劑".into();
                min_days_interval = 84; // 12 週
                acip_rule_summary = "衛福部 ACIP 規範：15歲以上3劑型，第 2 劑與第 3 劑最少需間隔 12 週 (84天)，且與第 1 劑隔至少 5 個月。".into();
            }
        }
        "shingles" => {
            vaccine_name = "非活性帶狀疱疹疫苗 (Shingrix)".into();
            next_dose_info = "第 2 劑 (自費/地方補助)".into();
            min_days_interval = 60; // 2 個月
            acip_rule_summary = "藥錄與ACIP規範：第 1 劑與第 2 劑建議間隔 2 至 6 個月 (60天至180天)。特殊免疫低下者最快可於1個月補打。".into();
            clinical_notes.push("建議於 2-6 個月黃金期內完成第 2 劑施打以達到最佳長期保護力。".into());
        }
        _ => {
            vaccine_name = "一般常規疫苗".into();
            next_dose_info = "下一劑".into();
            min_days_interval = 28;
            acip_rule_summary = "一般不活化疫苗最小間隔為 28 天 (4週)。".into();
        }
    }

    let earliest_date = last_date + chrono::Duration::days(min_days_interval);
    let days_remaining = (earliest_date - today).num_days();
    let is_ready_now = days_remaining <= 0;

    let earliest_date_display = format!(
        "{} 年 {} 月 {} 日",
        earliest_date.year(),
        earliest_date.month(),
        earliest_date.day()
    );

    Ok(CatchUpResponse {
        vaccine_name,
        next_dose_info,
        earliest_date_display,
        days_remaining,
        is_ready_now,
        acip_rule_summary,
        clinical_notes,
    })
}

#[derive(Serialize, Deserialize, Clone)]
struct TravelVaccineItem {
    name: String,
    requirement_type: String, // "Mandatory" | "Recommended" | "Booster"
    timing_note: String,
    yellow_book_required: bool,
    description: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct TravelAdvisoryResponse {
    destination_name: String,
    purpose_name: String,
    mandatory_items: Vec<TravelVaccineItem>,
    recommended_items: Vec<TravelVaccineItem>,
    booster_items: Vec<TravelVaccineItem>,
    travel_clinic_notes: Vec<String>,
}

#[tauri::command]
fn get_travel_advisory(
    destination: String,
    purpose: String,
) -> Result<TravelAdvisoryResponse, String> {
    let mut mandatory_items = Vec::new();
    let mut recommended_items = Vec::new();
    let mut booster_items = Vec::new();
    let mut travel_clinic_notes = Vec::new();

    let dest_name = match destination.as_str() {
        "all_global" => "🌍 全球通用 ‧ 國際預防接種、黃皮書與藥物綜合指南 (CDC 官方)",
        "us_eu_study" => "歐美留學 (美國 / 英國 / 加拿大 / 歐洲)",
        "japan_korea" => "日本 / 韓國 (觀光 / 留學 / 工作假期)",
        "southeast_asia" => "東南亞 / 南亞 (泰國 / 越南 / 印尼 / 印度 / 菲律賓)",
        "saudi_hajj" => "中東 / 沙烏地阿拉伯 (麥加朝聖 Hajj/Umrah / 工作)",
        "africa_latam" => "非洲 / 中南美洲 (黃熱病流行區 / 旅遊)",
        _ => "其他國際地區",
    }.to_string();

    let purpose_name = match purpose.as_str() {
        "study" => "🎓 留學 / 學校宿舍入住",
        "hajj" => "🕌 國際朝聖 / 宗教活動",
        "travel" => "✈️ 觀光旅遊 / 自由行",
        "work" => "💼 長期駐點 / 工作派遣",
        _ => "一般出國",
    }.to_string();

    match destination.as_str() {
        "all_global" => {
            mandatory_items.push(TravelVaccineItem {
                name: "黃熱病疫苗 (Yellow Fever)".into(),
                requirement_type: "Mandatory".into(),
                timing_note: "入境前至少 10 天施打 (終生有效)".into(),
                yellow_book_required: true,
                description: "依國際衛生條例 (IHR)，非洲及中南美洲黃熱病流行國強制要求出示國際預防接種證明書 (黃皮書) 入境簽證！未出示可能被拒絕入境。".into(),
            });
            mandatory_items.push(TravelVaccineItem {
                name: "四價腦膜炎雙球菌疫苗 (MenACWY)".into(),
                requirement_type: "Mandatory".into(),
                timing_note: "入境前至少 10 天於指定門診施打".into(),
                yellow_book_required: true,
                description: "沙烏地阿拉伯政府強制規定：赴麥加朝聖 (Hajj / Umrah) 入境簽證必須出示黃皮書證明！歐美大學宿舍亦列為強制入學項目。".into(),
            });
            mandatory_items.push(TravelVaccineItem {
                name: "小兒麻痺疫苗 (Polio IPV 追加劑)".into(),
                requirement_type: "Mandatory".into(),
                timing_note: "赴流行國(如阿富汗/巴基斯坦)停留4週以上者需於4週至12個月內追加1劑".into(),
                yellow_book_required: true,
                description: "疾管署規範：依 WHO 警告前往小兒麻痺野生株或衍生株高風險流行國，出境前必須出示黃皮書登載 1 年內 IPV 追加劑紀錄！".into(),
            });

            recommended_items.push(TravelVaccineItem {
                name: "傷寒疫苗 (Typhoid Vaccine)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "出發前 2 週施打 1 劑 (效期3年)".into(),
                yellow_book_required: false,
                description: "防範經由污染飲食與水質感染之傷寒桿菌，效期 3 年，持續高風險地區活動每 3 年建議追加 1 劑。".into(),
            });
            recommended_items.push(TravelVaccineItem {
                name: "A 型肝炎疫苗 (HepA)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "出發前 2 週施打第 1 劑 (隔6-12月打第2劑)".into(),
                yellow_book_required: false,
                description: "熱帶與開發中國家飲食衛生與水質感染風險高，建議預先接種 2 劑獲長期保護力。".into(),
            });
            recommended_items.push(TravelVaccineItem {
                name: "高山症預防用藥 (Acetazolamide / 丹木斯)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "登山前 24 小時開始服用至適應高海拔".into(),
                yellow_book_required: false,
                description: "赴海拔超過 2,500 公尺地區者 (秘魯庫斯科、西藏、高山健行)，建議至旅遊門診評估開立預防用藥。".into(),
            });

            travel_clinic_notes.push("抗瘧疾預防三大處方藥物：前往瘧疾流行區 (非洲/中南美/南亞)，請至旅遊門診評估開立：1) Malarone 莫可樂 (每日1顆)；2) Doxycycline 多喜黴素 (每日1顆)；3) Mefloquine 美爾奎寧 (每週1顆)。".into());
            travel_clinic_notes.push("黃熱病疫苗需於出發前至少 10 天於衛福部授權之旅遊醫學門診施打方能生效黃皮書。".into());
        }
        "us_eu_study" => {
            mandatory_items.push(TravelVaccineItem {
                name: "腦膜炎雙球菌疫苗 (MenACWY)".into(),
                requirement_type: "Mandatory".into(),
                timing_note: "入住宿舍前 2-4 週完成 (保護力 5 年)".into(),
                yellow_book_required: false,
                description: "美國、加拿大及歐洲絕大多數大學強制要求入住學校宿舍之學生出示 1 劑 MenACWY 接種證明。".into(),
            });
            mandatory_items.push(TravelVaccineItem {
                name: "麻疹腮腺炎德國麻疹疫苗 (MMR)".into(),
                requirement_type: "Mandatory".into(),
                timing_note: "入學前完成 2 劑紀錄或血清抗體陽性報告".into(),
                yellow_book_required: false,
                description: "歐美大學幾乎 100% 強制要求提交完整 2 劑 MMR 紀錄或抽血抗體證明。".into(),
            });
            mandatory_items.push(TravelVaccineItem {
                name: "水痘疫苗 (Varicella)".into(),
                requirement_type: "Mandatory".into(),
                timing_note: "入學前完成 2 劑紀錄或抗體報告".into(),
                yellow_book_required: false,
                description: "學校體檢表強制要求出示 2 劑水痘紀錄或幼時確診紀錄證明。".into(),
            });

            recommended_items.push(TravelVaccineItem {
                name: "B 型腦膜炎雙球菌疫苗 (MenB)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "出國前按 0, 1 個月施打 2 劑".into(),
                yellow_book_required: false,
                description: "防範 B 型腦膜炎雙球菌，歐美名校宿舍常強烈建議追加。".into(),
            });
            recommended_items.push(TravelVaccineItem {
                name: "九價人類乳突病毒疫苗 (HPV 9價)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "按 0, 2, 6 個月完成 3 劑".into(),
                yellow_book_required: false,
                description: "建議出國前完成全套 3 劑，海外醫療費用昂貴自費負擔重。".into(),
            });

            booster_items.push(TravelVaccineItem {
                name: "減量破傷風白喉百日咳疫苗 (Tdap)".into(),
                requirement_type: "Booster".into(),
                timing_note: "近 10 年內需追加 1 劑".into(),
                yellow_book_required: false,
                description: "學校體檢表通常要求提供過去 10 年內之 Tdap 追加紀錄。".into(),
            });

            travel_clinic_notes.push("建議提前 1 至 2 個月至旅遊醫學門診請醫師協助填寫並簽署國外大學英文體檢與疫苗表格 (Immunization Form)。".into());
        }
        "saudi_hajj" => {
            mandatory_items.push(TravelVaccineItem {
                name: "四價腦膜炎雙球菌疫苗 (MenACWY)".into(),
                requirement_type: "Mandatory".into(),
                timing_note: "入境前至少 10 天於指定門診施打".into(),
                yellow_book_required: true,
                description: "沙烏地阿拉伯政府強制規定：赴麥加朝聖 (Hajj / Umrah) 入境簽證必須出示黃皮書 (國際預防接種證明書)！".into(),
            });

            recommended_items.push(TravelVaccineItem {
                name: "A 型肝炎疫苗 (HepA)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "出發前 2 週施打第 1 劑 (隔6-12月打第2劑)".into(),
                yellow_book_required: false,
                description: "中東地區外食與水質風險，建議預先施打獲得保護力。".into(),
            });
            recommended_items.push(TravelVaccineItem {
                name: "季節性流感疫苗".into(),
                requirement_type: "Recommended".into(),
                timing_note: "出發前 2 週施打 1 劑".into(),
                yellow_book_required: false,
                description: "朝聖數百萬上百國人潮密集聚集，極易引發呼吸道傳染病爆發。".into(),
            });

            booster_items.push(TravelVaccineItem {
                name: "COVID-19 最新株疫苗".into(),
                requirement_type: "Booster".into(),
                timing_note: "依最新沙烏地入境規定提示施打".into(),
                yellow_book_required: false,
                description: "建議於出國前完成最新株疫苗追加。".into(),
            });

            travel_clinic_notes.push("前往沙烏地朝聖之民眾，必須至衛生福利部授權之「旅遊醫學門診」施打並領取蓋章之「國際預防接種證明書 (黃皮書)」。".into());
        }
        "africa_latam" => {
            mandatory_items.push(TravelVaccineItem {
                name: "黃熱病疫苗 (Yellow Fever)".into(),
                requirement_type: "Mandatory".into(),
                timing_note: "入境前至少 10 天施打 (終生有效)".into(),
                yellow_book_required: true,
                description: "依國際衛生條例 (IHR) 規定，非洲 (如肯亞、衣索比亞) 及中南美洲 (如巴西、祕魯、哥倫比亞) 特定國家強制要求出示國際預防接種證明書 (黃皮書) 入境簽證！未出示可能被拒絕入境或強制隔離。".into(),
            });

            recommended_items.push(TravelVaccineItem {
                name: "傷寒疫苗 (Typhoid Vaccine)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "出發前 2 週施打 1 劑 (效期3年)".into(),
                yellow_book_required: false,
                description: "防範經由污染飲食與水質感染之傷寒桿菌，效期 3 年，持續高風險地區活動每 3 年建議追加 1 劑。".into(),
            });
            recommended_items.push(TravelVaccineItem {
                name: "A 型肝炎疫苗 (HepA)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "出發前 2 週施打第 1 劑 (隔6-12月打第2劑)".into(),
                yellow_book_required: false,
                description: "熱帶疫區飲食衛生與水質感染風險高，建議預先接種 2 劑獲長期保護力。".into(),
            });
            recommended_items.push(TravelVaccineItem {
                name: "狂犬病疫苗 (Rabies)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "按 0, 7, 21-28 天完成 3 劑暴露前預防".into(),
                yellow_book_required: false,
                description: "前往野生動物或犬隻狂犬病高風險熱帶叢林野外活動建議預先接種。".into(),
            });
            recommended_items.push(TravelVaccineItem {
                name: "高山症預防用藥 (Acetazolamide / 丹木斯)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "登山前 24 小時開始服用至適應高海拔".into(),
                yellow_book_required: false,
                description: "赴秘魯庫斯科 (Cusco)、印加古道或衣索比亞高山等海拔超過 2,500 公尺地區者，建議至旅遊門診評估開立預防用藥。".into(),
            });

            travel_clinic_notes.push("抗瘧疾預防處方藥物：非洲與中南美洲多數地區有瘧疾流行，建議至旅遊門診開立預防藥物 (如 Malarone 莫可樂每日1顆，或 Doxycycline 每日1顆，或 Mefloquine 每週1顆) 於出發前開始服用。".into());
            travel_clinic_notes.push("黃熱病疫苗施打後需 10 天方能產生有效保護力並生效黃皮書，請務必提早預約。".into());
        }
        "southeast_asia" => {
            mandatory_items.push(TravelVaccineItem {
                name: "小兒麻痺疫苗 (Polio IPV 追加劑)".into(),
                requirement_type: "Mandatory".into(),
                timing_note: "赴流行國(如阿富汗/巴基斯坦)停留4週以上者需於4週至12個月內追加1劑".into(),
                yellow_book_required: true,
                description: "疾管署規範：依 WHO 警告前往小兒麻痺野生株或衍生株高風險流行國，出境前必須出示黃皮書登載 1 年內 IPV 追加劑紀錄！".into(),
            });

            recommended_items.push(TravelVaccineItem {
                name: "A 型肝炎疫苗 (HepA)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "出發前 2 週施打第 1 劑".into(),
                yellow_book_required: false,
                description: "東南亞 (泰國、越南、印尼、菲律賓) 飲食、路邊攤與冰塊水質 A 肝風險高，強烈建議自費接種。".into(),
            });
            recommended_items.push(TravelVaccineItem {
                name: "傷寒疫苗 (Typhoid Vaccine)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "出發前 2 週施打 1 劑 (效期3年)".into(),
                yellow_book_required: false,
                description: "赴印度、尼泊爾、東南亞偏遠地區，防範傷寒桿菌水質污染。".into(),
            });
            recommended_items.push(TravelVaccineItem {
                name: "麻疹腮腺炎德國麻疹疫苗 (MMR)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "出發前 2 週自費補打 1 劑".into(),
                yellow_book_required: false,
                description: "東南亞為麻疹流行高風險區，成人抗體衰退者建議出國前補打。".into(),
            });

            travel_clinic_notes.push("防蚊提醒：東南亞地區登革熱、屈公病及寨卡病毒活躍，請準備含 DEET 或 Picaridin 成分之有效防蚊液。".into());
            travel_clinic_notes.push("前往印度、印尼偏遠山區，建議諮詢旅遊門診評估開立抗瘧疾藥物 (Malarone 或 Doxycycline)。".into());
        }
        "japan_korea" => {
            recommended_items.push(TravelVaccineItem {
                name: "麻疹腮腺炎德國麻疹疫苗 (MMR)".into(),
                requirement_type: "Recommended".into(),
                timing_note: "出發前 2 週補打 1 劑".into(),
                yellow_book_required: false,
                description: "日本、韓國近年偶有麻疹境外與本土流行，1966年後出生之成人抗體多已衰退，建議赴日韓前補打 1 劑。".into(),
            });
            recommended_items.push(TravelVaccineItem {
                name: "季節性流感疫苗".into(),
                requirement_type: "Recommended".into(),
                timing_note: "秋冬季節出國前 2 週施打".into(),
                yellow_book_required: false,
                description: "日韓秋冬流感疫情嚴峻，出國前接種可大幅降低旅途生病風險。".into(),
            });

            booster_items.push(TravelVaccineItem {
                name: "A 型肝炎疫苗 (HepA)".into(),
                requirement_type: "Booster".into(),
                timing_note: "長期打工度假/留學/外食族建議打 2 劑".into(),
                yellow_book_required: false,
                description: "生食海鮮 (生魚片、生蠔) 頻率高者建議獲得 A 肝完整防護。".into(),
            });

            travel_clinic_notes.push("日韓醫療費用對外國旅客昂貴，建議出國前投保包含突發疾病醫療之海外旅行平安險。".into());
        }
        _ => {}
    }

    Ok(TravelAdvisoryResponse {
        destination_name: dest_name,
        purpose_name,
        mandatory_items,
        recommended_items,
        booster_items,
        travel_clinic_notes,
    })
}

#[derive(Serialize, Deserialize, Clone)]
struct GrowthMetricResult {
    metric_name: String,
    user_val: f64,
    unit: String,
    percentile_label: String,
    percentile_val: f64,
    p3: f64,
    p15: f64,
    p50: f64,
    p85: f64,
    p97: f64,
    status_summary: String,
    is_warning: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct GrowthResponse {
    age_display: String,
    gender_display: String,
    data_sources_citation: Vec<String>,
    height_result: GrowthMetricResult,
    weight_result: Option<GrowthMetricResult>,
    bmi_result: Option<GrowthMetricResult>,
    head_result: Option<GrowthMetricResult>,
    overall_advice: Vec<String>,
}

#[tauri::command]
fn calculate_growth_percentile(
    gender: String,
    age_months: i32,
    height: f64,
    weight: f64,
    head: Option<f64>,
) -> Result<GrowthResponse, String> {
    let is_male = gender == "male";
    let gender_display = if is_male { "男童 / 男青少年 ♂".to_string() } else { "女童 / 女青少年 ♀".to_string() };

    let years = age_months / 12;
    let rem_months = age_months % 12;

    let age_display = if age_months < 12 {
        format!("滿 {} 個月大", age_months)
    } else if rem_months == 0 {
        format!("滿 {} 歲大", years)
    } else {
        format!("{} 歲 {} 個月大", years, rem_months)
    };

    let mut data_sources_citation = Vec::new();
    data_sources_citation.push("0-5歲：衛福部國健署現行《兒童健康手冊》(2024最新版) 採用之 WHO 2006 國際生長標準 (2009國健署修訂公告)".to_string());
    data_sources_citation.push("5-7歲：衛福部國健署 2009 年公布之國人兒童生長銜接標準 (Chen & Chang)".to_string());
    data_sources_citation.push("7-18歲：衛福部國健署最新公告《兒童及青少年身體質量指數(BMI)與身高百分位建議值》(衛署授升字第0990700680號公告, 2010年)".to_string());

    // 1. 身高 / 身長參考數據 (0 ~ 18 歲)
    let (h_p3, h_p15, h_p50, h_p85, h_p97) = if years < 7 {
        // 0 ~ 7 歲（0~84 個月）插值計算
        if is_male {
            let base = 50.0 + (age_months as f64) * 0.95;
            (base - 4.5, base - 2.2, base, base + 2.5, base + 4.8)
        } else {
            let base = 49.0 + (age_months as f64) * 0.92;
            (base - 4.3, base - 2.1, base, base + 2.4, base + 4.6)
        }
    } else {
        // 7 ~ 18 歲：衛福部國健署 2010 年官方公告 Chen & Chang 身高參考值 (cm)
        let idx = (years.min(18) - 7) as usize;
        let male_height_table: [(f64, f64, f64, f64, f64); 12] = [
            (111.8, 115.4, 121.2, 126.9, 131.2), // 7歲
            (117.0, 120.7, 126.8, 132.8, 137.2), // 8歲
            (121.8, 125.7, 131.8, 138.2, 142.5), // 9歲
            (126.0, 130.4, 136.5, 143.2, 148.3), // 10歲
            (130.5, 135.5, 142.0, 149.3, 156.1), // 11歲
            (135.6, 141.5, 148.8, 156.6, 164.4), // 12歲
            (141.9, 148.7, 156.9, 164.6, 171.0), // 13歲
            (149.3, 155.8, 163.7, 170.8, 176.0), // 14歲
            (155.5, 161.2, 167.6, 173.8, 179.0), // 15歲
            (159.3, 164.5, 170.0, 176.0, 180.5), // 16歲
            (160.9, 166.0, 171.5, 177.2, 181.5), // 17歲
            (161.5, 166.5, 172.0, 177.8, 182.0), // 18歲
        ];
        let female_height_table: [(f64, f64, f64, f64, f64); 12] = [
            (110.6, 114.1, 120.3, 126.3, 130.1), // 7歲
            (115.7, 119.5, 125.8, 132.0, 136.5), // 8歲
            (120.7, 124.7, 131.3, 138.0, 143.5), // 9歲
            (125.8, 130.3, 137.5, 144.9, 150.8), // 10歲
            (131.8, 136.8, 144.5, 152.0, 157.3), // 11歲
            (137.9, 143.4, 150.5, 157.5, 161.8), // 12歲
            (143.2, 148.4, 154.5, 160.4, 164.8), // 13歲
            (146.8, 151.4, 156.8, 162.1, 167.0), // 14歲
            (148.5, 152.9, 157.9, 163.0, 168.2), // 15歲
            (149.5, 153.8, 158.7, 163.8, 168.8), // 16歲
            (150.0, 154.3, 159.3, 164.3, 169.0), // 17歲
            (150.0, 154.5, 159.5, 164.5, 169.0), // 18歲
        ];
        if is_male { male_height_table[idx] } else { female_height_table[idx] }
    };

    fn calc_metric(name: &str, val: f64, unit: &str, p3: f64, p15: f64, p50: f64, p85: f64, p97: f64) -> GrowthMetricResult {
        let (label, perc, summary, is_warn) = if val < p3 {
            ("低於 3rd 百分位".to_string(), 2.0, "生長偏矮/偏低（建議諮詢兒科）".to_string(), true)
        } else if val < p15 {
            ("3rd ~ 15th 百分位".to_string(), 10.0, "正常範圍（偏中下）".to_string(), false)
        } else if val < p50 {
            ("15th ~ 50th 百分位".to_string(), 35.0, "正常良好".to_string(), false)
        } else if val < p85 {
            ("50th ~ 85th 百分位".to_string(), 70.0, "正常良好".to_string(), false)
        } else if val < p97 {
            ("85th ~ 97th 百分位".to_string(), 90.0, "正常偏高".to_string(), false)
        } else {
            ("高於 97th 百分位".to_string(), 98.0, "身高生長超前".to_string(), false)
        };

        GrowthMetricResult {
            metric_name: name.to_string(),
            user_val: val,
            unit: unit.to_string(),
            percentile_label: label,
            percentile_val: perc,
            p3, p15, p50, p85, p97,
            status_summary: summary,
            is_warning: is_warn,
        }
    }

    let height_result = calc_metric("身長 / 身高", height, "cm", h_p3, h_p15, h_p50, h_p85, h_p97);

    // 2. 0~7 歲評估體重百分位
    let weight_result = if years < 7 {
        let (w_p3, w_p15, w_p50, w_p85, w_p97) = if is_male {
            let base = 3.3 + (age_months as f64) * 0.38;
            (base * 0.78, base * 0.88, base, base * 1.15, base * 1.28)
        } else {
            let base = 3.2 + (age_months as f64) * 0.35;
            (base * 0.77, base * 0.87, base, base * 1.14, base * 1.27)
        };
        Some(calc_metric("體重", weight, "kg", w_p3, w_p15, w_p50, w_p85, w_p97))
    } else {
        None
    };

    // 3. 7~18 歲依據衛福部國健署 2010 年《兒童及青少年BMI建議值》計算 BMI
    let bmi_result = if years >= 7 {
        let h_m = height / 100.0;
        let bmi_val = (weight / (h_m * h_m) * 10.0).round() / 10.0;
        let idx = (years.min(18) - 7) as usize;

        // 衛福部國健署 2010 官方公告 cut-off：(過輕, 正常中位數估計, 過重切點P85, 肥胖切點P95)
        let male_bmi_cutoffs: [(f64, f64, f64, f64); 12] = [
            (14.7, 16.5, 18.6, 21.2), // 7歲
            (15.0, 16.9, 19.3, 22.0), // 8歲
            (15.2, 17.3, 19.7, 22.5), // 9歲
            (15.4, 17.8, 20.3, 22.9), // 10歲
            (15.6, 18.2, 21.0, 23.5), // 11歲
            (15.9, 18.7, 21.5, 24.2), // 12歲
            (16.4, 19.3, 22.2, 24.8), // 13歲
            (17.0, 19.9, 22.7, 25.2), // 14歲
            (17.6, 20.4, 23.1, 25.5), // 15歲
            (18.2, 20.9, 23.4, 25.6), // 16歲
            (18.6, 21.3, 23.6, 25.6), // 17歲
            (19.0, 21.7, 23.7, 25.6), // 18歲
        ];

        let female_bmi_cutoffs: [(f64, f64, f64, f64); 12] = [
            (14.3, 16.0, 18.0, 20.3), // 7歲
            (14.5, 16.4, 18.8, 21.0), // 8歲
            (14.7, 16.8, 19.3, 21.6), // 9歲
            (15.0, 17.3, 20.1, 22.3), // 10歲
            (15.3, 17.8, 20.9, 23.1), // 11歲
            (15.7, 18.4, 21.6, 23.9), // 12歲
            (16.2, 18.9, 22.2, 24.6), // 13歲
            (16.7, 19.4, 22.7, 25.1), // 14歲
            (17.1, 19.7, 22.7, 25.3), // 15歲
            (17.4, 19.9, 22.7, 25.3), // 16歲
            (17.6, 20.1, 22.7, 25.3), // 17歲
            (17.8, 20.3, 22.7, 25.3), // 18歲
        ];

        let (b_p5, b_p50, b_p85, b_p95) = if is_male { male_bmi_cutoffs[idx] } else { female_bmi_cutoffs[idx] };

        let (label, perc, summary, is_warn) = if bmi_val < b_p5 {
            ("體重過輕 (低於 5th 百分位)".to_string(), 3.0, "體位過輕（建議營養諮詢）".to_string(), true)
        } else if bmi_val < b_p85 {
            ("體位適中正常 (5th ~ 85th 百分位)".to_string(), 50.0, "體位標準良好".to_string(), false)
        } else if bmi_val < b_p95 {
            ("體重過重 (85th ~ 95th 百分位)".to_string(), 88.0, "體位過重（建議飲食與運動調整）".to_string(), true)
        } else {
            ("肥胖 (高於 95th 百分位)".to_string(), 97.0, "體位肥胖（建議諮詢醫師評估）".to_string(), true)
        };

        Some(GrowthMetricResult {
            metric_name: "BMI 身體質量指數".to_string(),
            user_val: bmi_val,
            unit: "kg/m²".to_string(),
            percentile_label: label,
            percentile_val: perc,
            p3: b_p5,
            p15: b_p5,
            p50: b_p50,
            p85: b_p85,
            p97: b_p95,
            status_summary: summary,
            is_warning: is_warn,
        })
    } else {
        None
    };

    // 4. 頭圍評估 (僅 0~7 歲)
    let head_result = if years < 7 {
        head.map(|hd| {
            let (hd_p3, hd_p15, hd_p50, hd_p85, hd_p97) = if is_male {
                let base = 34.5 + (age_months as f64) * 0.18;
                (base - 2.0, base - 1.0, base, base + 1.1, base + 2.1)
            } else {
                let base = 34.0 + (age_months as f64) * 0.17;
                (base - 1.9, base - 0.9, base, base + 1.0, base + 2.0)
            };
            calc_metric("頭圍", hd, "cm", hd_p3, hd_p15, hd_p50, hd_p85, hd_p97)
        })
    } else {
        None
    };

    let mut overall_advice = Vec::new();
    overall_advice.push("衛生福利部國民健康署提醒：生長指標落於 3% 至 97% 之間（或 BMI 在 5%~85% 之間）皆屬正常健康生長範圍。".to_string());
    overall_advice.push("請持續觀察連續時間的「生長趨勢走勢」，比起單一時間點的數字，曲線平穩上升最為重要。".to_string());

    if height_result.is_warning || (bmi_result.as_ref().map_or(false, |b| b.is_warning)) {
        overall_advice.push("⚠️ 注意：若身高低於 3% 或 BMI 達到過重/肥胖界值，建議於門診時諮詢小兒內分泌科或兒科醫師進行專業兒童生長發展評估。".to_string());
    }

    Ok(GrowthResponse {
        age_display,
        gender_display,
        data_sources_citation,
        height_result,
        weight_result,
        bmi_result,
        head_result,
        overall_advice,
    })
}

#[tauri::command]
fn launch_external_calendar_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener().open_url(&url, None::<&str>).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "windows")]
    ensure_webview2_loader();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_eligible_vaccines,
            get_all_vaccines,
            calculate_catch_up,
            get_travel_advisory,
            calculate_growth_percentile,
            launch_external_calendar_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


