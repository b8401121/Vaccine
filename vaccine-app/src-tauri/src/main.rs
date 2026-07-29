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
    category: String,
    description: String,
    audience: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct TimelineMilestone {
    title: String,
    age_months: i32,
    status: String, // "Past", "Current", "Next"
    vaccines: Vec<VaccineItem>,
}

#[derive(Serialize, Deserialize, Clone)]
struct VaccineResponse {
    age_display: String,
    child_age_detail: String,
    milestones: Vec<TimelineMilestone>,
}

struct MilestoneSpec {
    title: &'static str,
    min_month: i32,
    max_month: i32,
    vaccines: Vec<VaccineItem>,
}

#[tauri::command]
fn get_eligible_vaccines(
    year: i32,
    month: u32,
    day: u32,
    is_roc: bool,
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

    let child_age_detail = if total_months <= 120 {
        if age_years > 0 {
            format!("{} 歲 {} 個月 (相當於 {} 個月大)", age_years, age_months, total_months)
        } else {
            format!("{} 個月大", age_months.max(0))
        }
    } else {
        format!("{} 歲", age_years)
    };

    let mut milestones_out = Vec::new();

    // 兒童垂直時間軸節點定義
    let child_specs = vec![
        MilestoneSpec {
            title: "出生 24 小時內",
            min_month: 0,
            max_month: 1,
            vaccines: vec![VaccineItem {
                name: "B 型肝炎疫苗".into(),
                dose_info: "第 1 劑".into(),
                timing_info: "出生 24 小時內".into(),
                category: "Routine".into(),
                description: "新生兒出生後儘速施打".into(),
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
                description: "基礎劑第 2 劑".into(),
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
                    description: "預防白喉、破傷風、百日咳、B型嗜血桿菌、小兒麻痺".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "13 價結合型肺炎鏈球菌疫苗 (PCV13)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 2 個月".into(),
                    category: "Routine".into(),
                    description: "基礎劑第 1 劑".into(),
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
                    description: "基礎劑第 2 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "13 價結合型肺炎鏈球菌疫苗 (PCV13)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 4 個月".into(),
                    category: "Routine".into(),
                    description: "基礎劑第 2 劑".into(),
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
                description: "建議於滿 5-8 個月施打".into(),
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
                    description: "基礎劑第 3 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "B 型肝炎疫苗".into(),
                    dose_info: "第 3 劑".into(),
                    timing_info: "滿 6 個月".into(),
                    category: "Routine".into(),
                    description: "基礎劑第 3 劑".into(),
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
                    description: "滿 12 個月施打 1 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "麻疹腮腺炎德國麻疹混合疫苗 (MMR)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 12 個月".into(),
                    category: "Routine".into(),
                    description: "滿 12 個月施打第 1 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "13 價結合型肺炎鏈球菌疫苗 (PCV13)".into(),
                    dose_info: "第 3 劑 (追加劑)".into(),
                    timing_info: "滿 12-15 個月".into(),
                    category: "Routine".into(),
                    description: "滿 12-15 個月施打追加劑".into(),
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
                description: "滿 15 個月施打第 1 劑".into(),
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
                    description: "滿 18 個月追加第 4 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "A 型肝炎疫苗 (Hep A)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 18 個月".into(),
                    category: "Routine".into(),
                    description: "滿 18 個月施打第 1 劑".into(),
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
                    description: "與第 1 劑隔至少 12 個月".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "A 型肝炎疫苗 (Hep A)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 27 個月".into(),
                    category: "Routine".into(),
                    description: "與第 1 劑隔至少 6 個月".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "滿 5 歲至國小入學前",
            min_month: 60,
            max_month: 85,
            vaccines: vec![
                VaccineItem {
                    name: "麻疹腮腺炎德國麻疹混合疫苗 (MMR)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 5 歲至國小入學前".into(),
                    category: "Routine".into(),
                    description: "入學前完成施打第 2 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "白喉破傷風百日咳及小兒麻痺疫苗 (DTaP-IPV)".into(),
                    dose_info: "追加劑 (第 5 劑)".into(),
                    timing_info: "滿 5 歲至國小入學前".into(),
                    category: "Routine".into(),
                    description: "入學前追加 1 劑".into(),
                    audience: "Children".into(),
                },
            ],
        },
    ];

    if total_months <= 120 {
        for spec in child_specs {
            let status = if total_months >= spec.max_month {
                "Past"
            } else if total_months >= spec.min_month && total_months < spec.max_month {
                "Current"
            } else {
                "Next"
            };

            milestones_out.push(TimelineMilestone {
                title: spec.title.to_string(),
                age_months: spec.min_month,
                status: status.to_string(),
                vaccines: spec.vaccines,
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
                description: "所有成人每年建議施打 1 劑".into(),
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
                dose_info: "第 1 劑 / 每10年追加".into(),
                timing_info: "成人常規".into(),
                category: "Routine".into(),
                description: "建議完成 1 劑 Tdap，之後每 10 年追加 1 劑".into(),
                audience: "Adults".into(),
            },
        ];

        if age_years >= 19 && age_years <= 45 {
            adult_routine.push(VaccineItem {
                name: "人類乳突病毒疫苗 (HPV)".into(),
                dose_info: "共 3 劑 (0-2-6 個月)".into(),
                timing_info: if age_years <= 26 { "19-26 歲常規" } else { "27-45 歲高風險" }.into(),
                category: if age_years <= 26 { "Routine".into() } else { "HighRisk".into() },
                description: "預防 HPV 感染引起之相關病變".into(),
                audience: "Adults".into(),
            });
        }

        if age_years >= 50 {
            adult_routine.push(VaccineItem {
                name: "非活性帶狀疱疹疫苗 (Shingles)".into(),
                dose_info: "共 2 劑 (隔 2-6 月)".into(),
                timing_info: "50 歲以上".into(),
                category: "Routine".into(),
                description: "預防帶狀疱疹及疱疹後神經痛".into(),
                audience: "Adults".into(),
            });
        }

        if age_years >= 65 || (age_years >= 55 && age_years <= 64) {
            adult_routine.push(VaccineItem {
                name: "肺炎鏈球菌疫苗 (PCV13/20 & PPV23)".into(),
                dose_info: "公費 1-2 劑".into(),
                timing_info: if age_years >= 65 { "65 歲以上" } else { "55-64 歲原住民" }.into(),
                category: "Routine".into(),
                description: "公費提供 1 劑 PCV20 或 PCV13 銜接 PPV23".into(),
                audience: "Adults".into(),
            });
        }

        if age_years >= 75 || age_years >= 60 {
            adult_routine.push(VaccineItem {
                name: "呼吸道細胞融合病毒 (RSV) 疫苗".into(),
                dose_info: "1 劑".into(),
                timing_info: if age_years >= 75 { "75 歲以上" } else { "60-74 歲高風險" }.into(),
                category: if age_years >= 75 { "Routine".into() } else { "HighRisk".into() },
                description: "預防 RSV 引發之下呼吸道疾病".into(),
                audience: "Adults".into(),
            });
        }

        milestones_out.push(TimelineMilestone {
            title: format!("成人常規與建議疫苗 ({})", age_display),
            age_months: total_months,
            status: "Current".to_string(),
            vaccines: adult_routine,
        });

        let adult_high_risk = vec![
            VaccineItem {
                name: "麻疹腮腺炎德國麻疹疫苗 (MMR)".into(),
                dose_info: "1-2 劑 (間隔28天以上)".into(),
                timing_info: "1966年後出生/無抗體者".into(),
                category: "HighRisk".into(),
                description: "育齡婦女、醫療人員或出國高風險對象補打".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "B 型肝炎疫苗".into(),
                dose_info: "共 3 劑 (按 0-1-6 月時程)".into(),
                timing_info: "抗體阴性/高風險".into(),
                category: "HighRisk".into(),
                description: "經檢驗為 B 肝抗體陰性者建議自費補打".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "A 型肝炎疫苗".into(),
                dose_info: "共 2 劑 (間隔6-12個月)".into(),
                timing_info: "高風險/赴流行區".into(),
                category: "HighRisk".into(),
                description: "慢性肝病或頻繁赴流行地區者".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "M 痘 (Mpox) 疫苗".into(),
                dose_info: "共 2 劑 (間隔4週)".into(),
                timing_info: "具風險行為者".into(),
                category: "HighRisk".into(),
                description: "暴露前/後預防與風險行為者施打".into(),
                audience: "Adults".into(),
            },
        ];

        milestones_out.push(TimelineMilestone {
            title: "特定對象與高風險評估疫苗".to_string(),
            age_months: total_months + 1,
            status: "Current".to_string(),
            vaccines: adult_high_risk,
        });
    }

    Ok(VaccineResponse {
        age_display,
        child_age_detail,
        milestones: milestones_out,
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_eligible_vaccines])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
