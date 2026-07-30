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
    category: String, // "Routine" (公費/常規) 或 "SelfPaid" (自費/建議)
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
    gender_display: String,
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
    gender: String,
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

    let is_female = gender == "female";
    let gender_display = if is_female { "女性 ♀" } else { "男性 ♂" }.to_string();

    let mut milestones_out = Vec::new();

    // 兒童及自費疫苗時間軸定義 (參考 CDC 自費與公費預防接種清單)
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
                    dose_info: "自費口服第 1 劑".into(),
                    timing_info: "滿 2 個月".into(),
                    category: "SelfPaid".into(),
                    description: "自費口服疫苗 (2劑型或3劑型)，預防輪狀病毒嚴重腹瀉".into(),
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
                    dose_info: "自費口服第 2 劑".into(),
                    timing_info: "滿 4 個月".into(),
                    category: "SelfPaid".into(),
                    description: "自費口服第 2 劑 (2劑型此劑完成，3劑型需於6個月再服)".into(),
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
            max_month: 85,
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

        if age_years >= 50 || age_years >= 18 {
            adult_routine.push(VaccineItem {
                name: "非活性帶狀疱疹疫苗 (Shingrix)".into(),
                dose_info: "自費共 2 劑 (隔 2-6 月)".into(),
                timing_info: "50 歲以上或 18 歲以上高風險".into(),
                category: "SelfPaid".into(),
                description: "預防帶狀疱疹(皮蛇)及疱疹後神經痛，防護率達90%以上".into(),
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

        milestones_out.push(TimelineMilestone {
            title: format!("成人常規與自費建議疫苗 ({}, {})", age_display, gender_display),
            age_months: total_months,
            status: "Current".to_string(),
            vaccines: adult_routine,
        });

        // 旅遊與特殊自費疫苗 (CDC 國際旅遊門診專區)
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

        milestones_out.push(TimelineMilestone {
            title: "旅遊醫學與特定自費疫苗 (CDC 清單)".to_string(),
            age_months: total_months + 1,
            status: "Current".to_string(),
            vaccines: travel_specs,
        });
    }

    Ok(VaccineResponse {
        age_display,
        child_age_detail,
        gender_display,
        milestones: milestones_out,
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_eligible_vaccines])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
