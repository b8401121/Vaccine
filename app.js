// Wasm is loaded lazily via dynamic import so that even if loading fails,
// the rest of the page (tabs, date selectors, etc.) still works.
let wasmModule = null;

async function loadWasm() {
  if (wasmModule) return wasmModule;
  try {
    const mod = await import('./wasm3/vaccine_core.js?t=' + Date.now());
    await mod.default(); // call initWasm
    wasmModule = mod;
    console.log('WebAssembly core initialized successfully.');
  } catch (e) {
    console.error('Failed to initialize WebAssembly:', e);
    throw e;
  }
  return wasmModule;
}

async function fallbackInvoke(cmd, args = {}) {
  const wasm = await loadWasm();
  
  if (cmd === 'get_eligible_vaccines') {
    const res = wasm.get_eligible_vaccines(args.year, args.month, args.day, args.isRoc, args.gender, args.location);
    return JSON.parse(res);
  }
  if (cmd === 'get_all_vaccines') {
    const res = wasm.get_all_vaccines();
    return JSON.parse(res);
  }
  if (cmd === 'calculate_catch_up') {
    const vaccine_id = args.vaccineId !== undefined ? args.vaccineId : args.vaccine_id;
    const last_dose_num = args.lastDoseNum !== undefined ? args.lastDoseNum : args.last_dose_num;
    const is_roc = args.isRoc !== undefined ? args.isRoc : args.is_roc;
    
    const res = wasm.calculate_catch_up(vaccine_id, last_dose_num, args.year, args.month, args.day, is_roc);
    return JSON.parse(res);
  }
  if (cmd === 'get_travel_advisory') {
    const res = wasm.get_travel_advisory(args.destination, args.purpose);
    return JSON.parse(res);
  }
  if (cmd === 'calculate_growth_percentile') {
    // Map the properties correctly since frontend uses camelCase/shorthand but WASM expects explicit args
    const age_months = args.ageMonths !== undefined ? args.ageMonths : args.age_months;
    const height_cm = args.height !== undefined ? args.height : args.height_cm;
    const weight_kg = args.weight !== undefined ? args.weight : args.weight_kg;
    const head_cm = args.head !== undefined ? args.head : args.head_cm;
    
    const res = wasm.calculate_growth_percentile(args.gender, age_months, height_cm, weight_kg, head_cm);
    return JSON.parse(res);
  }
  if (cmd === 'launch_external_calendar_url') {
    window.open(args.url, '_blank');
    return;
  }
  
  console.warn('Unknown Wasm invoke command:', cmd, args);
  throw new Error(`Command ${cmd} not implemented in WebAssembly`);
}

const invoke = (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) || 
               (window.__TAURI__ && window.__TAURI__.tauri && window.__TAURI__.tauri.invoke) || 
               (async (cmd, args) => {
  return await fallbackInvoke(cmd, args);
});

// Detect if running on Android/mobile Tauri
const isMobile = !!window.__TAURI_MOBILE__;

let allVaccinesList = [];
let currentFilter = 'all';
let lastQueryData = null;

// Navigation history stack for Android back button support
const tabHistoryStack = ['tab-calculator'];

window.addEventListener('DOMContentLoaded', () => {
  setupLoginSystem();
  
  setupDateSelectors();
  setupCatchupDateSelectors();
  setupCalendarToggle();
  setupFormSubmit();
  setupCatchupFormSubmit();
  setupTravelFormSubmit();
  setupGrowthFormSubmit();
  if (!isMobile) setupPrintButton();
  setupTabs();
  setupLibraryFilterAndSearch();
  setupModalEvents();
  setupCalendarModalEvents();
  setupAndroidBackButtonHandler();

  // On mobile, hide all print-related elements
  if (isMobile) {
    document.querySelectorAll(
      '#print-report-btn, #printable-report, #print-select-modal, .print-btn'
    ).forEach(el => { el.style.display = 'none'; });
    // Add mobile class to body for CSS targeting
    document.body.classList.add('is-mobile');
  }

  // 預先載入疫苗圖鑑庫
  loadVaccineLibrary();
});

// Android 手機返回鍵監聽處理
function setupAndroidBackButtonHandler() {
  // 推入初始歷史紀錄，讓 window.onpopstate 能攔截返回鍵
  history.pushState({ tab: 'tab-calculator', hasResults: false }, '');

  window.addEventListener('popstate', (e) => {
    // 1. 優先檢查：若有開著的 Modal 彈窗，優先關閉彈窗
    const openModals = document.querySelectorAll('.modal-overlay:not(.hidden)');
    if (openModals.length > 0) {
      openModals.forEach(m => m.classList.add('hidden'));
      history.pushState({ tab: tabHistoryStack[tabHistoryStack.length - 1] || 'tab-calculator' }, '');
      return;
    }

    const currentTab = tabHistoryStack[tabHistoryStack.length - 1] || 'tab-calculator';

    // 2. 若目前在首頁 (tab-calculator) 且已往下捲動 (例如看時間軸或結果)
    if (currentTab === 'tab-calculator') {
      const resultsDiv = document.getElementById('results');
      const hasVisibleResults = resultsDiv && !resultsDiv.classList.contains('hidden');
      
      if (window.scrollY > 80 || hasVisibleResults) {
        // 先向上捲動回頂部表單區
        window.scrollTo({ top: 0, behavior: 'smooth' });
        // 若歷史堆疊有多個重複的 tab-calculator，只保留一個
        while (tabHistoryStack.length > 1 && tabHistoryStack[tabHistoryStack.length - 1] === 'tab-calculator') {
          tabHistoryStack.pop();
        }
        history.pushState({ tab: 'tab-calculator' }, '');
        return;
      }

      // 如果頁籤堆疊還有其他分頁 (如之前去過大百科)，退回該分頁
      if (tabHistoryStack.length > 1) {
        tabHistoryStack.pop();
        const previousTabId = tabHistoryStack[tabHistoryStack.length - 1];
        switchTab(previousTabId, false);
        history.pushState({ tab: previousTabId }, '');
        return;
      }

      // 已在首頁最頂部且無其他歷史，允許退出程式
      return;
    }

    // 3. 若在其他分頁 (大百科、補打試算、出國速查)，退回上一頁籤
    if (tabHistoryStack.length > 1) {
      tabHistoryStack.pop();
      const previousTabId = tabHistoryStack[tabHistoryStack.length - 1];
      switchTab(previousTabId, false);
      history.pushState({ tab: previousTabId }, '');
    }
  });
}

function switchTab(targetId, pushHistory = true) {
  const tabBtns = document.querySelectorAll('.tab-btn');
  tabBtns.forEach(b => {
    if (b.getAttribute('data-target') === targetId) {
      b.classList.add('active');
    } else {
      b.classList.remove('active');
    }
  });

  document.querySelectorAll('.tab-page').forEach(page => {
    page.classList.add('hidden');
    page.classList.remove('active');
  });

  const activePage = document.getElementById(targetId);
  if (activePage) {
    activePage.classList.remove('hidden');
    activePage.classList.add('active');
  }

  if (pushHistory) {
    if (tabHistoryStack[tabHistoryStack.length - 1] !== targetId) {
      tabHistoryStack.push(targetId);
      history.pushState({ tab: targetId }, '');
    }
  }
}



function setupDateSelectors() {
  const yearInput = document.getElementById('year');
  const monthSelect = document.getElementById('month');
  const daySelect = document.getElementById('day');

  const now = new Date();
  yearInput.value = now.getFullYear();

  for (let m = 1; m <= 12; m++) {
    const opt = document.createElement('option');
    opt.value = m;
    opt.textContent = `${m} 月`;
    if (m === now.getMonth() + 1) opt.selected = true;
    monthSelect.appendChild(opt);
  }

  function updateDays() {
    const year = parseInt(yearInput.value) || 2000;
    const month = parseInt(monthSelect.value) || 1;
    const isRoc = document.getElementById('calendar-toggle').checked;
    const actualYear = isRoc ? year + 1911 : year;

    const daysInMonth = new Date(actualYear, month, 0).getDate();
    const currentSelectedDay = parseInt(daySelect.value) || now.getDate();

    daySelect.innerHTML = '';
    for (let d = 1; d <= daysInMonth; d++) {
      const opt = document.createElement('option');
      opt.value = d;
      opt.textContent = `${d} 日`;
      if (d === currentSelectedDay || (d === daysInMonth && currentSelectedDay > daysInMonth)) {
        opt.selected = true;
      }
      daySelect.appendChild(opt);
    }
  }

  yearInput.addEventListener('input', updateDays);
  monthSelect.addEventListener('change', updateDays);
  updateDays();
}

function setupCalendarToggle() {
  const toggle = document.getElementById('calendar-toggle');
  const text = document.getElementById('calendar-type-text');
  const yearInput = document.getElementById('year');

  toggle.addEventListener('change', (e) => {
    const now = new Date();
    if (e.target.checked) {
      text.textContent = '民國年 (ROC)';
      yearInput.placeholder = '如: 80';
      yearInput.value = now.getFullYear() - 1911;
    } else {
      text.textContent = '西元年 (AD)';
      yearInput.placeholder = '如: 1991';
      yearInput.value = now.getFullYear();
    }
    const event = new Event('input');
    yearInput.dispatchEvent(event);
  });
}

function setupFormSubmit() {
  const form = document.getElementById('dob-form');
  const yearInput = document.getElementById('year');
  const monthSelect = document.getElementById('month');
  const daySelect = document.getElementById('day');
  const calendarToggle = document.getElementById('calendar-toggle');

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const year = parseInt(yearInput.value);
    const month = parseInt(monthSelect.value);
    const day = parseInt(daySelect.value);
    const isRoc = calendarToggle.checked;
    const gender = document.querySelector('input[name="gender"]:checked').value;
    const location = document.getElementById('location').value;

    try {
      const response = await invoke('get_eligible_vaccines', { year, month, day, isRoc, gender, location });
      displayVaccines(response);
    } catch (error) {
      alert(`錯誤: ${error}`);
    }
  });

  const resetBtn = document.getElementById('reset-form-btn');
  if (resetBtn) {
    resetBtn.addEventListener('click', () => {
      // 恢復西元年與目前年份
      calendarToggle.checked = false;
      const text = document.getElementById('calendar-type-text');
      if (text) text.textContent = '西元年 (AD)';
      
      const now = new Date();
      yearInput.placeholder = '如: 1990';
      yearInput.value = now.getFullYear();
      monthSelect.value = now.getMonth() + 1;
      
      // 更新日期下拉選單並重設女性、預設縣市
      const event = new Event('input');
      yearInput.dispatchEvent(event);
      daySelect.value = now.getDate();

      const femaleRadio = document.querySelector('input[name="gender"][value="female"]');
      if (femaleRadio) femaleRadio.checked = true;

      const locationSelect = document.getElementById('location');
      if (locationSelect) locationSelect.value = '桃園市';

      // 隱藏查詢結果區塊並捲動回頂部
      const resultsDiv = document.getElementById('results');
      if (resultsDiv) resultsDiv.classList.add('hidden');
      const fluBannerContainer = document.getElementById('flu-rec-banner-container');
      if (fluBannerContainer) fluBannerContainer.innerHTML = '';
      lastQueryData = null;

      window.scrollTo({ top: 0, behavior: 'smooth' });
    });
  }

  const jumpBtn = document.getElementById('jump-to-current');
  if (jumpBtn) {
    jumpBtn.addEventListener('click', scrollToCurrentNode);
  }
}

function renderFluAgeRecommendation(ageText) {
  const container = document.getElementById('flu-rec-banner-container');
  if (!container) return;

  // 解析年齡文字 (如: "0歲2個月", "1歲6個月", "2歲3個月", "4歲", "25歲", "55歲", "72歲")
  let ageYears = 0;
  let ageMonths = 0;

  const yMatch = ageText.match(/(\d+)\s*歲/);
  const mMatch = ageText.match(/(\d+)\s*個?月/);

  if (yMatch) ageYears = parseInt(yMatch[1], 10);
  if (mMatch) ageMonths = parseInt(mMatch[1], 10);
  const totalMonths = ageYears * 12 + ageMonths;

  let title = '';
  let badge = '2026 流感選用指南';
  let bodyHtml = '';
  let brandPills = [];

  if (totalMonths < 6) {
    title = '🍂 2026 流感防護提醒：未滿 6 個月嬰兒尚未達流感疫苗施打年齡';
    badge = '同住家人防護';
    bodyHtml = `
      <p>依據衛福部適應症規定，目前所有市售流感疫苗均僅適用於<strong>滿 6 個月以上</strong>幼兒。</p>
      <p style="margin-top:0.35rem; color:#b91c1c;"><strong>💡 醫師專業建議：</strong>為保護未滿半歲小寶寶，強烈建議<strong>父母、祖父母及同住照顧者</strong>務必全員接種流感疫苗，並落實洗手與佩戴口罩，形成家庭隱形防護罩！</p>
    `;
    brandPills = ['同住家人接種', '包被策略', '滿6個月後施打'];
  } else if (totalMonths < 24) {
    // 6個月 ~ 未滿2歲
    title = '🍂 2026 流感疫苗首選推薦 (滿6個月~未滿2歲幼兒)';
    badge = '標準四價型 / 無蛋細胞培養';
    bodyHtml = `
      <p>寶寶已滿 6 個月，進入流感重症高危險群！建議儘速施打流感疫苗。</p>
      <p style="margin-top:0.35rem;"><strong>💉 適用廠牌：</strong>東洋 輔流威護 (MDCK細胞培養/無蛋蛋白/吻合度高)、GSK 伏流感 (德國雞胚)、賽諾菲 菲流達 (法國雞胚)。</p>
      <p style="margin-top:0.35rem; color:#c2410c;"><strong>⚠️ 劑次提醒：</strong>8 歲以下「初次」接種流感疫苗之幼兒，需接種 <strong>2 劑</strong>（兩劑間隔至少 4 週），方能誘發足量抗體；若往年已接種過則僅需打 1 劑。</p>
    `;
    brandPills = ['東洋 輔流威護(細胞無蛋)', 'GSK 伏流感', '賽諾菲 菲流達', '初次需打2劑'];
  } else if (totalMonths < 36) {
    // 2歲 ~ 未滿3歲
    title = '🍂 2026 流感疫苗首選推薦 (2歲~未滿3歲幼童)';
    badge = '一般注射 或 自費鼻噴免打針';
    bodyHtml = `
      <p>滿 2 歲幼童可選擇傳統肌肉注射或<strong>唯一免打針無痛鼻噴式疫苗</strong>！</p>
      <p style="margin-top:0.35rem;"><strong>💉 標準注射推薦：</strong>東洋 輔流威護 (細胞培養)、GSK 伏流感、賽諾菲 菲流達。</p>
      <p style="margin-top:0.35rem; color:#b45309;"><strong>👃 害怕打針兒少首選：</strong>可自費選用 <strong>AZ 能伏鼻 (FluMist)</strong> 鼻噴式活性減毒疫苗！免打針無疼痛，直接在鼻黏膜形成第一線 IgA 防禦抗體。<br><small style="color:#ef4444;">(禁忌注意：嚴重免疫缺損、重度氣喘或近4週曾有喘鳴、長期服用阿斯匹靈者禁忌使用)</small></p>
    `;
    brandPills = ['AZ 能伏鼻(鼻噴免打針)', '東洋 輔流威護', 'GSK 伏流感', '賽諾菲 菲流達'];
  } else if (ageYears < 18) {
    // 3歲 ~ 17歲
    title = '🍂 2026 流感疫苗首選推薦 (3歲~17歲兒童與青少年)';
    badge = '校園公費主力 或 鼻噴免打針';
    bodyHtml = `
      <p>3 歲以上所有 8 大流感疫苗中已有 6 款適用（含國光安定伏、高端福喜健），校園公費與自費皆可配合施打。</p>
      <p style="margin-top:0.35rem;"><strong>💉 適用廠牌：</strong>國光 安定伏、高端 福喜健、東洋 輔流威護 (細胞培養)、GSK 伏流感、賽諾菲 菲流達。</p>
      <p style="margin-top:0.35rem; color:#b45309;"><strong>👃 免打針自費推薦：</strong>害怕打針的學童強烈推薦自費 <strong>AZ 能伏鼻 (FluMist)</strong> 鼻噴式活性減毒疫苗，無痛提升呼吸道黏膜免疫！</p>
    `;
    brandPills = ['AZ 能伏鼻(鼻噴無痛)', '國光 安定伏', '高端 福喜健', '東洋 輔流威護', 'GSK 伏流感', '賽諾菲 菲流達'];
  } else if (ageYears < 50) {
    // 18歲 ~ 49歲
    title = '🍂 2026 流感疫苗首選推薦 (18~49歲青壯年)';
    badge = '標準四價型 / 形成群聚防護';
    bodyHtml = `
      <p>青壯年為家庭經濟支柱與社會主要活動者，建議每年接種 1 劑流感疫苗，降低流感傳染給家中幼童與長輩風險。</p>
      <p style="margin-top:0.35rem;"><strong>💉 推薦標準型：</strong>東洋 輔流威護 (細胞培養/精準防護)、GSK 伏流感、賽諾菲 菲流達、國光 安定伏、高端 福喜健。</p>
      <p style="margin-top:0.35rem; color:#0369a1;"><strong>💡 貼心提醒：</strong>公費符合資格（如高風險慢性病、孕婦、6個月內嬰兒之父母）請按時公費施打；非公費者可自費預約施打。</p>
    `;
    brandPills = ['東洋 輔流威護(細胞型)', 'GSK 伏流感', '賽諾菲 菲流達', '國光 安定伏', '高端 福喜健'];
  } else if (ageYears < 65) {
    // 50歲 ~ 64歲
    title = '🍂 2026 流感疫苗首選推薦 (50~64歲熟齡族群)';
    badge = '強烈推薦：東洋 輔流禦 (MF59佐劑加強型)';
    bodyHtml = `
      <p>50 歲起人體免疫力隨年齡逐步下降（免疫老化）。研究顯示，一般標準型流感疫苗在熟齡長者體內抗體衰退速度較快。</p>
      <p style="margin-top:0.35rem; color:#c2410c;"><strong>🔥 首選加強型：</strong>強烈建議自費預約選用【<strong>東洋 輔流禦 (Fluad) 佐劑加強型</strong>】！其添加專利 MF59 佐劑，可強效活化體內免疫系統，提供長達 <strong>12 個月</strong>的持久抗體防禦。</p>
      <p style="margin-top:0.35rem;"><strong>💉 標準型選擇：</strong>亦可選擇東洋輔流威護、GSK伏流感、賽諾菲菲流達、國光安定伏、高端福喜健等標準型公費/自費疫苗。</p>
    `;
    brandPills = ['⭐ 首選：東洋 輔流禦(佐劑加強型)', '東洋 輔流威護', 'GSK 伏流感', '賽諾菲 菲流達', '公費50歲起施打'];
  } else {
    // 65歲以上
    title = '🍂 2026 流感疫苗長者專用推薦 (65歲以上銀髮長輩)';
    badge = '長者專用加強型：菲優達高劑量 / 輔流禦佐劑型';
    bodyHtml = `
      <p>65 歲以上為流感併發重症、肺炎住院及心血管死亡之最高風險族群！國際醫學指引高度建議長者優先施打<strong>加強型流感疫苗</strong>。</p>
      <p style="margin-top:0.35rem; color:#b91c1c;"><strong>🏆 兩大長者專用加強型：</strong></p>
      <ul style="margin: 0.35rem 0 0.5rem 1.25rem; font-size: 0.9rem; line-height: 1.6;">
        <li><strong>賽諾菲 菲優達 (Efluelda High-Dose)</strong>：含 4 倍抗原量，臨床實證重症防護力提升 <strong>24.2%</strong>，大幅降低心肺併發症住院率。</li>
        <li><strong>東洋 輔流禦 (Fluad Tetra)</strong>：添加 MF59 專利佐劑，抗體效價顯著提高且保護力長達 12 個月。</li>
      </ul>
      <p style="margin-top:0.25rem; color:#475569;">（公費安養機構長照長者享有專案配送加強型；社區長者亦可諮詢自費升級預約；標準型公費亦享有免費施打）</p>
    `;
    brandPills = ['⭐ 賽諾菲 菲優達(4倍高劑量)', '⭐ 東洋 輔流禦(佐劑加強型)', '防範重症住院+24.2%', '公費標準型免費提供'];
  }

  const pillsHtml = brandPills.map(p => `<span class="flu-brand-pill">${p}</span>`).join('');

  container.innerHTML = `
    <div class="flu-rec-banner fade-in">
      <div class="flu-rec-banner-header">
        <div class="flu-rec-title">
          <span>🍂</span>
          <span>${title}</span>
        </div>
        <span class="flu-rec-badge">${badge}</span>
      </div>
      <div class="flu-rec-content">
        ${bodyHtml}
      </div>
      <div class="flu-rec-brands-list">
        ${pillsHtml}
      </div>
    </div>
  `;
}

function scrollToCurrentNode() {
  const currentNode = document.querySelector('.current-node');
  if (currentNode) {
    currentNode.scrollIntoView({ behavior: 'smooth', block: 'center' });
  } else {
    const results = document.getElementById('results');
    if (results) {
      results.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }
}

// 依年齡產生對應的流感疫苗卡片物件 (支援直接放入時間軸與列印)
function getFluVaccineItemsForAge(ageYears, totalMonths) {
  if (totalMonths < 6) {
    return []; // 未滿6個月不符合施打條件
  }

  if (totalMonths < 24) {
    // 6個月~未滿2歲
    return [
      {
        name: "季節性流感疫苗 (滿6個月~未滿2歲幼兒推薦)",
        dose_info: "第 1-2 劑 (8歲以下初次施打需隔4週打2劑)",
        timing_info: "每年秋冬流感季 (滿6個月以上)",
        category: "Routine",
        description: "【滿6個月~未滿2歲】：建議施打標準型四價疫苗。首選【東洋 輔流威護(MDCK犬腎細胞培養/無蛋/吻合度最高)】、GSK 伏流感(德國)、賽諾菲 菲流達(法國)。8歲以下初次接種需打2劑。",
        audience: "Children"
      }
    ];
  } else if (totalMonths < 36) {
    // 2歲~未滿3歲
    return [
      {
        name: "季節性流感疫苗 (2歲~未滿3歲幼童推薦)",
        dose_info: "每年 1-2 劑 (初次需隔4週打2劑)",
        timing_info: "每年秋冬流感季",
        category: "Routine",
        description: "【2歲~未滿3歲幼童】：標準注射可選東洋輔流威護(細胞型)、GSK伏流感、賽諾菲菲流達。◆ 害怕打針者首選：自費 AZ 能伏鼻 (FluMist 唯一免打針鼻噴式活性減毒，鼻黏膜第一線IgA防衛，注意氣喘喘鳴禁忌)。",
        audience: "Children"
      }
    ];
  } else if (ageYears < 18) {
    // 3歲~17歲
    return [
      {
        name: "季節性流感疫苗 (3歲~17歲兒少推薦)",
        dose_info: "每年 1 劑 (未滿9歲初次打2劑)",
        timing_info: "每年秋冬流感季 (校園公費/自費)",
        category: "Routine",
        description: "【3歲~17歲兒童青少年】：標準注射適用國光安定伏、高端福喜健、東洋輔流威護、GSK伏流感、賽諾菲菲流達。◆ 害怕打針者首選：自費 AZ 能伏鼻 (FluMist 唯一免打針無痛鼻噴疫苗)。",
        audience: "Children"
      }
    ];
  } else if (ageYears < 50) {
    // 18~49歲
    return [
      {
        name: "季節性流感疫苗 (18~49歲成人標準防護)",
        dose_info: "每年 1 劑",
        timing_info: "秋冬流感季 (18-49歲一般成人)",
        category: "Routine",
        description: "【18~49歲青壯年】：可選擇標準型疫苗（東洋 輔流威護【細胞培養/無蛋】、GSK 伏流感、賽諾菲 菲流達、國光 安定伏、高端 福喜健）。公費資格者按時程施打，非公費者建議自費接種形成群體防護。",
        audience: "Adults"
      }
    ];
  } else if (ageYears < 65) {
    // 50~64歲
    return [
      {
        name: "季節性流感疫苗 (50~64歲熟齡佐劑加強型首選)",
        dose_info: "每年 1 劑",
        timing_info: "秋冬流感季 (50-64歲熟齡/免疫低下)",
        category: "Routine",
        description: "【50~64歲熟齡與免疫低下推薦】：建議選用【含佐劑加強型 東洋輔流禦 (Fluad)】，添加 MF59 專利佐劑能克服免疫老化，刺激更強且持久達12個月之抗體保護力。亦可選標準型（東洋輔流威護、GSK伏流感、賽諾菲菲流達、國光安定伏、高端福喜健）。",
        audience: "Adults"
      }
    ];
  } else {
    // 65歲以上
    return [
      {
        name: "季節性流感疫苗 (65歲以上銀髮長者加強型首選)",
        dose_info: "每年 1 劑",
        timing_info: "秋冬流感季 (65歲以上長者)",
        category: "Routine",
        description: "【65歲以上銀髮長者首選】：強烈建議選用加強型以克服免疫老化！首選【高劑量加強型 賽諾菲菲優達 (Efluelda High-Dose，含4倍抗原，重症保護力提升24.2%)】或【含佐劑加強型 東洋輔流禦 (Fluad)】。安養長照長者公費優先，一般長者可自費預約。亦可選一般標準型。",
        audience: "Adults"
      }
    ];
  }
}

function displayVaccines(data) {
  lastQueryData = data;
  const { age_display, child_age_detail, gender_display, location_display, current_visit_date, current_visit_milestone, next_visit_date, next_visit_milestone, milestones } = data;
  const resultsDiv = document.getElementById('results');
  const timelineContainer = document.getElementById('timeline-container');
  const ageBadge = document.getElementById('age-badge');

  const sumCurrentMilestone = document.getElementById('summary-current-milestone');
  const sumCurrentDate = document.getElementById('summary-current-date');
  const sumNextMilestone = document.getElementById('summary-next-milestone');
  const sumNextDate = document.getElementById('summary-next-date');

  if (sumCurrentMilestone) sumCurrentMilestone.textContent = `階段：${current_visit_milestone || '當前可施打'}`;
  if (sumCurrentDate) sumCurrentDate.textContent = `📅 建議接種日期：${current_visit_date || '即日起符合'}`;
  if (sumNextMilestone) sumNextMilestone.textContent = `階段：${next_visit_milestone || '無'}`;
  if (sumNextDate) sumNextDate.textContent = `📅 預估接種日期：${next_visit_date || '定期保養'}`;

  const ageText = child_age_detail || age_display;
  const fullMetaText = `🏙️ ${location_display} | ${gender_display} | 目前計算年齡：${ageText}`;

  ageBadge.textContent = fullMetaText;

  // 渲染 2026 流感疫苗動態適應症推薦卡片 (頂部摘要橫幅)
  renderFluAgeRecommendation(ageText);

  // 計算年齡數值
  let ageYears = 0;
  let ageMonths = 0;
  const yMatch = ageText.match(/(\d+)\s*歲/);
  const mMatch = ageText.match(/(\d+)\s*個?月/);
  if (yMatch) ageYears = parseInt(yMatch[1], 10);
  if (mMatch) ageMonths = parseInt(mMatch[1], 10);
  const totalMonths = ageYears * 12 + ageMonths;

  // 取得該年齡對應之流感疫苗卡片
  const fluItems = getFluVaccineItemsForAge(ageYears, totalMonths);

  // 確保「當前推薦站點」(Current) 或首個有效站點最前方包含該年齡之專屬流感疫苗卡片
  if (milestones && milestones.length > 0 && fluItems.length > 0) {
    // 先移除任何舊的流感項目，重新置頂插入最新的流感規格卡片
    let currentOrTargetNode = milestones.find(m => m.status === 'Current') || milestones.find(m => m.status === 'Next') || milestones[0];
    
    // 清除既有流感項目避免重複
    milestones.forEach(m => {
      if (m.vaccines) {
        m.vaccines = m.vaccines.filter(v => !v.name.includes('流感'));
      }
    });

    // 將流感疫苗卡片以 unshift 插入目標站點的最前端，確保成為第一張顯眼卡片
    if (currentOrTargetNode && currentOrTargetNode.vaccines) {
      currentOrTargetNode.vaccines.unshift(...fluItems);
    }

    // 遍歷所有站點，確保含有流感或口服疫苗時均有清晰部位與同次接種指引
    milestones.forEach(m => {
      if (!m.co_admin_guide) m.co_admin_guide = [];
      const mAgeMonths = m.age_months !== undefined ? m.age_months : totalMonths;
      const hasFlu = m.vaccines && m.vaccines.some(v => v.name.includes('流感'));
      const hasOral = m.vaccines && m.vaccines.some(v => v.name.includes('輪狀病毒'));

      if (hasFlu && !m.co_admin_guide.some(g => g.includes('流感'))) {
        if (mAgeMonths >= 6 && mAgeMonths < 24) {
          m.co_admin_guide.push(
            "🍂 流感疫苗部位指南 (滿6個月~未滿2歲)：流感疫苗(不活化針劑)可與同次五合一、B肝或13價肺炎鏈球菌同時施打於『不同側大腿』或同側大腿距離至少 2.5cm 處。"
          );
        } else if (mAgeMonths >= 24 && mAgeMonths < 216) {
          m.co_admin_guide.push(
            "🍂 流感疫苗部位指南 (2~17歲兒少)：可選傳統上臂三角肌注射，或自費 AZ 能伏鼻 (FluMist 唯一雙側鼻孔黏膜噴入免打針活性減毒，若與常規針劑同日施打免受扎針之苦；若與水痘/MMR非同日施打需間隔28天)。"
          );
        } else {
          m.co_admin_guide.push(
            "🍂 流感疫苗部位指南 (成人/長者)：流感疫苗與新冠疫苗、PCV20肺炎鏈球菌或破傷風Tdap，建議『分左右兩上手臂三角肌』同次同時接種，安全省時抗體互不干擾。"
          );
        }
      }

      if (hasOral && !m.co_admin_guide.some(g => g.includes('口服'))) {
        m.co_admin_guide.push(
          "🍼 口服減毒疫苗指南：口服輪狀病毒疫苗經腸道吸收，與肌肉注射之針劑(流感/五合一/肺炎)作用途徑完全不同，建議於針劑注射前或注射安撫後口服投藥，兩者不互斥。"
        );
      }
    });
  }

  // 渲染「當次建議接種 (Current Visit)」與「下次預計接種 (Next Visit)」卡片內的疫苗清單與流感專屬醒目標記
  const sumCurrentVaccinesEl = document.getElementById('summary-current-vaccines');
  const sumNextVaccinesEl = document.getElementById('summary-next-vaccines');
  
  if (sumCurrentVaccinesEl) {
    const currentMilestoneObj = milestones ? (milestones.find(m => m.status === 'Current') || milestones[0]) : null;
    let vaxListHtml = '';
    if (currentMilestoneObj && currentMilestoneObj.vaccines && currentMilestoneObj.vaccines.length > 0) {
      currentMilestoneObj.vaccines.forEach(v => {
        const isFlu = v.name.includes('流感');
        if (isFlu) {
          vaxListHtml += `<span class="visit-card-vax-pill visit-card-flu-pill"><span class="flu-dot"></span>🍂 ${v.name} (${v.dose_info})</span>`;
        } else {
          vaxListHtml += `<span class="visit-card-vax-pill">💉 ${v.name}</span>`;
        }
      });
    } else if (fluItems.length > 0) {
      vaxListHtml = `<span class="visit-card-vax-pill visit-card-flu-pill"><span class="flu-dot"></span>🍂 ${fluItems[0].name}</span>`;
    } else {
      vaxListHtml = `<span style="font-size:0.85rem; color:#64748b;">依診所排程接種常規疫苗</span>`;
    }
    sumCurrentVaccinesEl.innerHTML = vaxListHtml;
  }

  if (sumNextVaccinesEl) {
    const nextMilestoneObj = milestones ? milestones.find(m => m.status === 'Next') : null;
    let nextVaxHtml = '';
    if (nextMilestoneObj && nextMilestoneObj.vaccines && nextMilestoneObj.vaccines.length > 0) {
      nextMilestoneObj.vaccines.slice(0, 4).forEach(v => {
        nextVaxHtml += `<span class="visit-card-vax-pill">💉 ${v.name}</span>`;
      });
      if (nextMilestoneObj.vaccines.length > 4) {
        nextVaxHtml += `<span class="visit-card-vax-pill" style="color:#64748b;">+${nextMilestoneObj.vaccines.length - 4} 項</span>`;
      }
    } else {
      nextVaxHtml = `<span style="font-size:0.85rem; color:#64748b;">定期追蹤健康與疫苗接種</span>`;
    }
    sumNextVaccinesEl.innerHTML = nextVaxHtml;
  }

  timelineContainer.innerHTML = '';

  if (!milestones || milestones.length === 0) {
    timelineContainer.innerHTML = '<p class="empty" style="color: #94a3b8;">目前無特定時間軸資料。</p>';
  } else {
    milestones.forEach(m => {
      const node = document.createElement('div');
      
      let statusClass = 'next-node';
      let nodeIcon = '⏳';
      let statusLabel = '未來預計';

      if (m.status === 'Past') {
        statusClass = 'past-node';
        nodeIcon = '✓';
        statusLabel = '歷史已過期點';
      } else if (m.status === 'Current') {
        statusClass = 'current-node';
        nodeIcon = '📍';
        statusLabel = '當前推薦站點';
      }

      node.className = `timeline-item fade-in ${statusClass}`;

      let cardsHtml = '';
      m.vaccines.forEach(v => {
        let tagClass = 'routine';
        let tagText = '公費常規';
        if (v.category === 'Subsidized') {
          tagClass = 'subsidized';
          tagText = '🏛️ 地方縣市補助';
        } else if (v.category === 'SelfPaid') {
          tagClass = 'self-paid';
          tagText = '💰 自費建議';
        } else if (v.category === 'HighRisk') {
          tagClass = 'high-risk';
          tagText = '高風險對象';
        }

        const isCurrent = m.status === 'Current';
        const cardHighlightClass = isCurrent ? 'current-vaccine-card' : '';
        const currentBadge = isCurrent ? '<span class="due-now-badge">🔥 目前應施打</span>' : '';

        cardsHtml += `
          <div class="timeline-vaccine-card ${cardHighlightClass} clickable-vaccine-card"
               data-vaccine-name="${v.name.replace(/"/g, '&quot;')}"
               title="點擊查看 ${v.name} 完整解說">
            <div class="card-header">
              <h4>${v.name}</h4>
              <span class="tag ${tagClass}">${tagText}</span>
            </div>
            <div class="meta-badges">
              <span class="dose-badge">💉 ${v.dose_info}</span>
              <span class="timing-badge">📅 ${v.timing_info}</span>
              ${currentBadge}
            </div>
            <p class="dose-desc">${v.description}</p>
            <div class="vaccine-card-click-hint">🔍 點擊查看完整疫苗解說</div>
          </div>
        `;
      });

      const currentAgeBannerHtml = m.status === 'Current'
        ? `<div class="standalone-current-age-banner">📍 目前計算年齡：<strong>${child_age_detail || age_display}</strong></div>`
        : '';

      let coAdminHtml = '';
      if (m.co_admin_guide && m.co_admin_guide.length > 0) {
        coAdminHtml = `
          <div class="co-admin-box">
            <div class="co-admin-header">
              <span>💉</span>
              <strong>同次同時接種組合與施打部位指南 (Co-administration Guide)：</strong>
            </div>
            <ul class="co-admin-list">
              ${m.co_admin_guide.map(guide => `<li>${guide}</li>`).join('')}
            </ul>
          </div>
        `;
      }

      // 在時間軸框框內顯示「當次接種日期」或「下次預估日期」
      let visitDateBarHtml = '';
      if (m.status === 'Current') {
        const todayStr = (() => {
          const t = new Date();
          return `${t.getFullYear()}-${String(t.getMonth() + 1).padStart(2, '0')}-${String(t.getDate()).padStart(2, '0')}`;
        })();
        visitDateBarHtml = `
          <div class="milestone-visit-bar current-visit-bar">
            <span class="visit-bar-icon">📍</span>
            <span class="visit-bar-label">當次建議接種日期：</span>
            <strong class="visit-bar-date">${todayStr}（今日）</strong>
          </div>
        `;
      } else if (m.status === 'Next' && m.target_date && m.target_date.length === 10) {
        visitDateBarHtml = `
          <div class="milestone-visit-bar next-visit-bar">
            <span class="visit-bar-icon">📆</span>
            <span class="visit-bar-label">預估下次接種日期：</span>
            <strong class="visit-bar-date">${m.target_date}</strong>
          </div>
        `;
      }

      const inlineCurrentPrintBtn = m.status === 'Current'
        ? `<button class="inline-timeline-print-btn">🖨️ 列印 / 匯出本次衛教單</button>`
        : '';

      node.innerHTML = `
        ${currentAgeBannerHtml}
        <div class="timeline-marker">
          <div class="marker-dot">${nodeIcon}</div>
        </div>
        <div class="timeline-content">
          <div class="timeline-header">
            <div class="timeline-header-title">
              <h3 class="milestone-title">${m.title}</h3>
              <span class="status-pill ${statusClass}-pill">${statusLabel}</span>
            </div>
            <div class="timeline-header-actions">
              ${inlineCurrentPrintBtn}
              <button class="add-cal-btn" data-title="${m.title}" data-date="${m.target_date || ''}" data-vaccines="${m.vaccines.map(v => v.name).join('、')}">
                📱 📅 手機掃碼行事曆
              </button>
            </div>
          </div>
          ${visitDateBarHtml}
          <div class="timeline-cards-grid">
            ${cardsHtml}
          </div>
          ${coAdminHtml}
        </div>
      `;
        timelineContainer.appendChild(node);
    });

    // 時間軸疫苗卡片點擊 → 開啟疫苗解說 Modal
    timelineContainer.querySelectorAll('.clickable-vaccine-card').forEach(card => {
      card.addEventListener('click', () => {
        const name = card.getAttribute('data-vaccine-name');
        openVaccineModalByName(name, card);
      });
    });

    document.querySelectorAll('.add-cal-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const title = btn.getAttribute('data-title');
        const targetDate = btn.getAttribute('data-date');
        const vaccines = btn.getAttribute('data-vaccines');
        
        const today = new Date();
        const todayStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;
        const dateToUse = (targetDate && targetDate.length === 10) ? targetDate : todayStr;

        openCalendarModal(`預防接種提醒 — ${title}`, dateToUse, `建議接種疫苗：${vaccines}`);
      });
    });

    // 時間軸當次框框內「列印/匯出衛教單」按鈕
    timelineContainer.querySelectorAll('.inline-timeline-print-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        if (!lastQueryData) return;
        openPrintSelectModal(lastQueryData);
      });
    });
  }

  resultsDiv.classList.remove('hidden');
  setTimeout(scrollToCurrentNode, 200);
}

// ----------------------------------------------------
// 分頁 2：疫苗圖鑑庫 (Library View & Modal)
// ----------------------------------------------------
function setupTabs() {
  const tabBtns = document.querySelectorAll('.tab-btn');
  tabBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      const targetId = btn.getAttribute('data-target');
      switchTab(targetId, true);
    });
  });
}

// 2026 台灣流感疫苗 8 大廠牌完整資料庫（確保前端與 Wasm 雙模皆完整呈現）
const FLU_2026_VACCINES = [
  {
    id: "flu_guide_2026",
    name: "🍂 2026 台灣流感疫苗選用攻略 (8大廠牌總覽)",
    aliases: "2026流感疫苗 / 四價流感 / 雞胚細胞佐劑鼻噴 / 8大廠牌PK",
    category: "Both",
    target_audience: "滿 6 個月以上幼兒、孕婦、學童、成人至高齡長者 (全齡)",
    prevent_disease: "預防 2026-2027 年度世界衛生組織 (WHO) 推薦之四價流感病毒株（A型 H1N1、H3N2 及 B型 Victoria、Yamagata）",
    full_description: "2026年台灣市售流感疫苗共計 8 大廠牌，技術全面升級！涵蓋傳統雞胚培養、MDCK細胞培養、長者專用佐劑加強型、長者專用高劑量加強型，以及兒童青少年專用鼻噴式活性減毒疫苗。本指南依年齡、體質與預算提供專業適應症分析。",
    schedule: [
      "◆ 滿6個月~未滿9歲且初次施打：需接種 2 劑 (間隔 4 週以上)",
      "◆ 9歲以上至成人長者：每年秋冬定期接種 1 劑",
      "◆ 2歲~未滿18歲兒少：可選 AZ 能伏鼻 (唯一免打針鼻噴劑型)",
      "◆ 50歲以上或免疫低下者：推薦東洋 輔流禦 (MF59佐劑加強型)",
      "◆ 65歲以上銀髮長者：推薦賽諾菲 菲優達 (4倍高劑量加強型)"
    ],
    notes: "雞蛋過敏者可安心施打，亦可優先選用東洋輔流威護(細胞培養)。流感疫苗可與新冠、肺炎鏈球菌疫苗同日不同部位同時施打。"
  },
  {
    id: "flu_flucelvax",
    name: "東洋 輔流威護 四價流感疫苗 (Flucelvax Tetra)",
    aliases: "輔流威護 / 東洋細胞流感 / MDCK 細胞培養流感疫苗 / CSL Seqirus",
    category: "Both",
    target_audience: "滿 6 個月以上全齡通用 (嬰幼兒、孕婦、成人、長者)",
    prevent_disease: "季節性 A/B 型流感病毒感染及重症",
    full_description: "【MDCK犬腎細胞培養技術】完全不經過受精雞胚培養，零雞蛋蛋白殘留，徹底杜絕病毒在雞蛋培養過程中發生的突變適應，WHO臨床證實抗原吻合度最高！適合重度雞蛋過敏者或追求精準保護力之民眾。",
    schedule: [
      "滿6個月以上~未滿9歲且初次接種：施打 2 劑 (間隔 4 週)",
      "9歲以上與成人：每年接種 1 劑 (公費依配送/自費可指定)"
    ],
    notes: "公自費皆有供應。注射部位偶有輕微紅腫疼痛，多於2-3天內緩解。"
  },
  {
    id: "flu_fluad",
    name: "東洋 輔流禦 佐劑加強型四價流感疫苗 (Fluad Tetra)",
    aliases: "輔流禦 / Fluad / MF59佐劑流感疫苗 / 熟齡長者加強型",
    category: "Both",
    target_audience: "50 歲以上熟齡長者 / 服用免疫抑制劑者 / 免疫功能低下者",
    prevent_disease: "50歲以上族群季節性流感重症、肺炎住院及併發症",
    full_description: "【專利 MF59 免疫佐劑】添加專利角鯊烯水包油乳化佐劑，可大幅活化抗原呈現細胞與T/B淋巴細胞，專門克服長者與慢性病患之「免疫老化 (Immunosenescence)」，抗體保護力顯著增強且持久長達 12 個月！公費優先提供安養及長照機構65歲以上長者。",
    schedule: [
      "50歲以上成人：每年秋冬接種 1 劑 (肌肉注射 0.5mL)",
      "安養長照機構65歲以上長者享公費；一般50歲以上民眾可自費預約"
    ],
    notes: "適應症為50歲以上。局部紅腫酸痛比例略高於一般劑型，屬正常免疫反應。"
  },
  {
    id: "flu_efluelda",
    name: "賽諾菲 菲優達 高劑量四價流感疫苗 (Efluelda High-Dose)",
    aliases: "菲優達 / 賽諾菲高劑量流感 / Efluelda / Fluzone High-Dose",
    category: "Both",
    target_audience: "65 歲以上銀髮長者專用",
    prevent_disease: "高齡長者流感病毒感染、重症肺炎、心血管併發症及住院",
    full_description: "【4倍抗原高劑量加強型】每劑每株含有 60µg 血球凝集素 (HA)，總抗原量為一般標準疫苗 (15µg) 的 4 倍！大規模國際臨床實證，針對 65 歲以上長者之流感重症預防效果較標準型顯著提升 24.2%，大幅降低長者心肺併發症與住院率！",
    schedule: [
      "65歲以上長者：每年秋冬接種 1 劑 (肌肉注射 0.7mL)",
      "安養長照機構65歲以上長者公費配發；一般社區長者可自費預約施打"
    ],
    notes: "專為65歲以上長者設計，未滿65歲不適用。"
  },
  {
    id: "flu_flumist",
    name: "AZ 能伏鼻 鼻噴式活性減毒流感疫苗 (FluMist / Fluenz Tetra)",
    aliases: "能伏鼻 / FluMist / 鼻噴流感疫苗 / 免打針流感疫苗 / 阿斯特捷利康",
    category: "SelfPaid",
    target_audience: "2 歲至未滿 18 歲兒童與青少年 (全自費)",
    prevent_disease: "兒童與青少年流感感染、呼吸道併發症及群聚傳播",
    full_description: "【唯一免打針無痛流感疫苗】經雙側鼻孔黏膜各噴入 0.1mL 活性減毒疫苗。模仿病毒自然感染途徑，在鼻咽呼吸道黏膜直接刺激產生強效第一線分泌型 IgA 黏膜抗體與全身性細胞免疫！免除針頭恐懼，是兒童與青少年自費首選。",
    schedule: [
      "2歲至未滿9歲初次接種流感疫苗者：間隔至少 4 週噴入 2 劑",
      "曾接種過流感疫苗或9歲以上：每年施打 1 劑 (雙鼻孔各0.1mL)"
    ],
    notes: "⚠️【禁忌症 (不可接種)】：懷孕婦女、嚴重免疫缺陷者、長期服用阿斯匹靈(Aspirin)之兒少、重度氣喘或近4週內曾有喘鳴(wheezing)發作者。全自費，無公費。"
  },
  {
    id: "flu_fluarix",
    name: "GSK 伏流感 四價流感疫苗 (Fluarix Tetra)",
    aliases: "伏流感 / GSK 流感疫苗 / 葛蘭素史克",
    category: "Both",
    target_audience: "滿 6 個月以上全齡通用 (嬰幼兒、學童、成人、長者)",
    prevent_disease: "A 型與 B 型流感重症與併發症",
    full_description: "【傳統雞胚培養裂解疫苗】由全球疫苗領導大廠葛蘭素史克 (GSK) 於德國/歐洲原廠生產製造。全球使用量大、臨床研究歷史悠久，安全性與有效性廣受各國衛生當局認證。",
    schedule: [
      "滿6個月至未滿9歲初次接種：施打 2 劑 (間隔 4 週)",
      "9歲以上與成人：每年施打 1 劑"
    ],
    notes: "公費常規主力廠牌之一，自費門診亦常備。"
  },
  {
    id: "flu_vaxigrip",
    name: "賽諾菲 菲流達 四價流感疫苗 (Vaxigrip Tetra)",
    aliases: "菲流達 / 賽諾菲流感疫苗 / 巴斯德流感疫苗",
    category: "Both",
    target_audience: "滿 6 個月以上全齡通用 (嬰幼兒、孕婦、成人、長者)",
    prevent_disease: "A 型 (H1N1/H3N2) 及 B 型流感病毒引起的流行性感冒",
    full_description: "【法國賽諾菲原裝進口】歐洲巴斯德研發中心專業製造，為台灣歷年公費採購與小兒自費門診極為信賴之標準流感疫苗廠牌，孕婦及6個月大以上嬰兒均可安心接種。",
    schedule: [
      "滿6個月至未滿9歲初次接種：施打 2 劑 (間隔 4 週)",
      "9歲以上與成人：每年施打 1 劑"
    ],
    notes: "公費常規主力廠牌之一，自費可指定。"
  },
  {
    id: "flu_adimflu",
    name: "國光生技 安定伏 裂解型四價流感疫苗 (AdimFlu-S)",
    aliases: "安定伏 / 國光流感疫苗",
    category: "Both",
    target_audience: "滿 3 歲以上幼童、學童、青少年、成人與長者",
    prevent_disease: "預防 A/B 型流行性感冒病毒引發之感染與重症",
    full_description: "【台灣國產流感疫苗主力】由國光生物科技於台灣在地 PIC/S GMP 廠製造，供應台灣公費防疫計畫逾半數劑量。技術成熟、品質穩定，適用於 3 歲以上民眾。",
    schedule: [
      "滿3歲至未滿9歲且初次接種：施打 2 劑 (間隔 4 週)",
      "9歲以上至成人長者：每年施打 1 劑"
    ],
    notes: "注意：本品適應症為滿 3 歲以上，未滿 3 歲嬰幼兒請選用6個月適應症之廠牌。"
  },
  {
    id: "flu_gcflu",
    name: "高端 福喜健 四價流感疫苗 (GC Flu Quadrivalent)",
    aliases: "福喜健 / 高端流感疫苗 / GC Biopharma",
    category: "Both",
    target_audience: "滿 3 歲以上幼童、學童、青少年、成人與長者",
    prevent_disease: "預防季節性 A 型與 B 型流行性感冒病毒感染",
    full_description: "【與韓國生技大廠 GC Biopharma 合作】引進世界衛生組織 (WHO) 預認證之四價流感抗原原液，符合國際規格，適用於滿 3 歲以上之公自費流感接種。",
    schedule: [
      "滿3歲至未滿9歲初次接種：施打 2 劑 (間隔 4 週)",
      "9歲以上至成人長者：每年施打 1 劑"
    ],
    notes: "適應症為滿 3 歲以上，未滿 3 歲嬰幼兒請選擇適用月齡廠牌。"
  }
];

async function loadVaccineLibrary() {
  try {
    const wasmList = await invoke('get_all_vaccines');
    // 合併清單並依 ID 去重
    const mergedMap = new Map();
    if (Array.isArray(wasmList)) {
      wasmList.forEach(v => mergedMap.set(v.id || v.name, v));
    }
    FLU_2026_VACCINES.forEach(v => {
      mergedMap.set(v.id, v);
    });
    allVaccinesList = Array.from(mergedMap.values());
    renderLibraryGrid(allVaccinesList);
  } catch (err) {
    console.error('載入疫苗圖鑑庫失敗，使用內建資料庫:', err);
    allVaccinesList = FLU_2026_VACCINES;
    renderLibraryGrid(allVaccinesList);
  }
}

function setupLibraryFilterAndSearch() {
  const searchInput = document.getElementById('library-search');
  const chips = document.querySelectorAll('.chip');

  if (searchInput) {
    searchInput.addEventListener('input', () => filterAndRenderLibrary());
  }

  chips.forEach(chip => {
    chip.addEventListener('click', () => {
      chips.forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
      currentFilter = chip.getAttribute('data-filter');
      filterAndRenderLibrary();
    });
  });
}

function filterAndRenderLibrary() {
  const query = document.getElementById('library-search').value.toLowerCase().trim();

  const filtered = allVaccinesList.filter(item => {
    const matchesSearch = item.name.toLowerCase().includes(query) ||
                          item.aliases.toLowerCase().includes(query) ||
                          item.prevent_disease.toLowerCase().includes(query) ||
                          item.full_description.toLowerCase().includes(query);

    let matchesCategory = true;
    if (currentFilter === 'flu2026') {
      matchesCategory = item.id.startsWith('flu_') || item.name.includes('流感');
    } else if (currentFilter === 'Routine') {
      matchesCategory = item.category === 'Routine' || item.category === 'Both';
    } else if (currentFilter === 'SelfPaid') {
      matchesCategory = item.category === 'SelfPaid' || item.category === 'Both';
    } else if (currentFilter === 'child') {
      matchesCategory = item.target_audience.includes('兒童') || item.target_audience.includes('幼兒') || item.target_audience.includes('全齡');
    } else if (currentFilter === 'adult') {
      matchesCategory = item.target_audience.includes('成人') || item.target_audience.includes('長者') || item.target_audience.includes('熟齡') || item.target_audience.includes('全齡');
    }

    return matchesSearch && matchesCategory;
  });

  renderLibraryGrid(filtered);
}

function renderLibraryGrid(vaccines) {
  const grid = document.getElementById('library-grid');
  if (!grid) return;

  grid.innerHTML = '';

  if (vaccines.length === 0) {
    grid.innerHTML = '<p class="empty" style="grid-column: 1/-1; text-align: center; color: #94a3b8; padding: 2rem;">未找到符合條件的疫苗。</p>';
    return;
  }

  vaccines.forEach(v => {
    const card = document.createElement('div');
    card.className = 'library-card fade-in';

    let catBadgeClass = 'routine';
    let catBadgeText = '公費常規';
    if (v.category === 'SelfPaid') {
      catBadgeClass = 'self-paid';
      catBadgeText = '💰 自費建議';
    } else if (v.category === 'Both') {
      catBadgeClass = 'both-cat';
      catBadgeText = '公費 / 自費';
    }

    card.innerHTML = `
      <div class="library-card-header">
        <span class="tag ${catBadgeClass}">${catBadgeText}</span>
        <span class="target-tag">🎯 ${v.target_audience}</span>
      </div>
      <h3 class="library-card-title">${v.name}</h3>
      <p class="library-aliases">${v.aliases}</p>
      <div class="prevent-disease-box">
        <strong>🛡️ 預防疾病：</strong>
        <p>${v.prevent_disease}</p>
      </div>
      <p class="library-desc-preview">${v.full_description.substring(0, 75)}...</p>
      <button class="btn-detail-open">查看完整介紹與注射時程 ➔</button>
    `;

    card.addEventListener('click', () => openVaccineModal(v));
    grid.appendChild(card);
  });
}

function openVaccineModal(v) {
  const modal = document.getElementById('vaccine-modal');
  const modalContent = document.getElementById('modal-content');

  let scheduleListHtml = '';
  v.schedule.forEach(s => {
    scheduleListHtml += `<li>💉 ${s}</li>`;
  });

  let catBadgeText = v.category === 'Routine' ? '公費常規' : (v.category === 'SelfPaid' ? '💰 自費建議' : '公費 / 自費提供');

  modalContent.innerHTML = `
    <div class="modal-header-section">
      <span class="modal-category-tag">${catBadgeText}</span>
      <h2>${v.name}</h2>
      <p class="modal-aliases">${v.aliases}</p>
    </div>

    <div class="modal-body-section">
      <div class="modal-block">
        <h3>🛡️ 預防疾病與感染</h3>
        <p class="disease-text">${v.prevent_disease}</p>
      </div>

      <div class="modal-block">
        <h3>📖 疫苗簡介與作用</h3>
        <p class="desc-text">${v.full_description}</p>
      </div>

      <div class="modal-block schedule-block">
        <h3>📅 建議注射時程與劑次</h3>
        <ul class="schedule-list">
          ${scheduleListHtml}
        </ul>
      </div>

      <div class="modal-block notes-block">
        <h3>⚠️ 接種注意事項與禁忌</h3>
        <p class="notes-text">${v.notes}</p>
      </div>
    </div>
  `;

  modal.classList.remove('hidden');
}

// 從時間軸疫苗卡片點擊後呼叫：先找圖鑑，找不到則用卡片現有資料建簡易 Modal
function openVaccineModalByName(name, cardEl) {
  // 1. 嘗試在圖鑑中找完整資料
  if (allVaccinesList && allVaccinesList.length > 0) {
    const found = allVaccinesList.find(v =>
      v.name === name ||
      v.name.includes(name) ||
      name.includes(v.name) ||
      (v.aliases && v.aliases.toLowerCase().includes(name.toLowerCase()))
    );
    if (found) {
      openVaccineModal(found);
      return;
    }
  }

  // 2. Fallback：用時間軸卡片既有資料組合簡易 Modal
  const modal = document.getElementById('vaccine-modal');
  const modalContent = document.getElementById('modal-content');
  if (!modal || !modalContent) return;

  const doseText = cardEl.querySelector('.dose-badge')?.textContent || '';
  const timingText = cardEl.querySelector('.timing-badge')?.textContent || '';
  const descText = cardEl.querySelector('.dose-desc')?.textContent || '';
  const tagText = cardEl.querySelector('.tag')?.textContent || '';

  modalContent.innerHTML = `
    <div class="modal-header-section">
      <span class="modal-category-tag">${tagText}</span>
      <h2>${name}</h2>
      <p class="modal-aliases">（詳細圖鑑資料請至「疫苗百科圖鑑」頁籤查詢）</p>
    </div>
    <div class="modal-body-section">
      <div class="modal-block">
        <h3>💉 劑次與時程資訊</h3>
        <p class="desc-text">${doseText}　${timingText}</p>
      </div>
      <div class="modal-block">
        <h3>📖 本次衛教說明</h3>
        <p class="desc-text">${descText || '請參閱診所衛教人員說明。'}</p>
      </div>
      <div class="modal-block notes-block">
        <h3>⚠️ 接種注意事項</h3>
        <p class="notes-text">接種前請告知醫護人員是否有發燒、急性疾病、藥物過敏或免疫功能問題。接種後請留觀 15–30 分鐘。</p>
      </div>
    </div>
  `;
  modal.classList.remove('hidden');
}

function setupModalEvents() {
  const modal = document.getElementById('vaccine-modal');
  const closeBtn = document.getElementById('modal-close');

  if (closeBtn) {
    closeBtn.addEventListener('click', () => modal.classList.add('hidden'));
  }

  if (modal) {
    modal.addEventListener('click', (e) => {
      if (e.target === modal) {
        modal.classList.add('hidden');
      }
    });
  }

  // 2026 流感疫苗 8 大廠牌比較表 Modal 事件綁定
  const fluModal = document.getElementById('flu2026-modal');
  const fluOpenBtn = document.getElementById('open-flu2026-modal-btn');
  const fluCloseBtn = document.getElementById('flu2026-modal-close');

  if (fluOpenBtn && fluModal) {
    fluOpenBtn.addEventListener('click', () => {
      fluModal.classList.remove('hidden');
    });
  }

  if (fluCloseBtn && fluModal) {
    fluCloseBtn.addEventListener('click', () => {
      fluModal.classList.add('hidden');
    });
  }

  if (fluModal) {
    fluModal.addEventListener('click', (e) => {
      if (e.target === fluModal) {
        fluModal.classList.add('hidden');
      }
    });
  }
}

// ----------------------------------------------------
// 分頁 3：遲打 / 補打最短間隔試算器 (Catch-up Calculator)
// ----------------------------------------------------
function setupCatchupDateSelectors() {
  const yearInput = document.getElementById('catchup-year');
  const monthSelect = document.getElementById('catchup-month');
  const daySelect = document.getElementById('catchup-day');

  if (!yearInput || !monthSelect || !daySelect) return;

  const now = new Date();
  yearInput.value = now.getFullYear();

  for (let m = 1; m <= 12; m++) {
    const opt = document.createElement('option');
    opt.value = m;
    opt.textContent = `${m} 月`;
    if (m === now.getMonth() + 1) opt.selected = true;
    monthSelect.appendChild(opt);
  }

  function updateDays() {
    const year = parseInt(yearInput.value) || 2024;
    const month = parseInt(monthSelect.value) || 1;
    const daysInMonth = new Date(year, month, 0).getDate();
    const currentSelectedDay = parseInt(daySelect.value) || now.getDate();

    daySelect.innerHTML = '';
    for (let d = 1; d <= daysInMonth; d++) {
      const opt = document.createElement('option');
      opt.value = d;
      opt.textContent = `${d} 日`;
      if (d === currentSelectedDay || (d === daysInMonth && currentSelectedDay > daysInMonth)) {
        opt.selected = true;
      }
      daySelect.appendChild(opt);
    }
  }

  yearInput.addEventListener('input', updateDays);
  monthSelect.addEventListener('change', updateDays);
  updateDays();
}

function setupCatchupFormSubmit() {
  const form = document.getElementById('catchup-form');
  if (!form) return;

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const vaccineId = document.getElementById('catchup-vaccine').value;
    const lastDoseNum = parseInt(document.getElementById('catchup-dose').value);
    const year = parseInt(document.getElementById('catchup-year').value);
    const month = parseInt(document.getElementById('catchup-month').value);
    const day = parseInt(document.getElementById('catchup-day').value);

    try {
      const res = await invoke('calculate_catch_up', {
        vaccineId,
        lastDoseNum,
        year,
        month,
        day,
        isRoc: false
      });
      displayCatchupResult(res);
    } catch (err) {
      alert(`試算失敗: ${err}`);
    }
  });
}

function displayCatchupResult(data) {
  const container = document.getElementById('catchup-result');
  if (!container) return;

  container.classList.remove('hidden');

  const { vaccine_name, next_dose_info, earliest_date_display, days_remaining, is_ready_now, acip_rule_summary, clinical_notes } = data;

  let statusBadgeHtml = '';
  if (is_ready_now) {
    statusBadgeHtml = `<div class="catchup-status-badge ready">✅ 已符合 ACIP 最小間隔，目前隨時可補打接種！</div>`;
  } else {
    statusBadgeHtml = `<div class="catchup-status-badge waiting">⏳ 尚需等待：倒數 ${days_remaining} 天 (未滿最短間隔時間)</div>`;
  }

  let notesHtml = '';
  if (clinical_notes && clinical_notes.length > 0) {
    notesHtml = `
      <div class="catchup-notes-box">
        <h4 style="margin-bottom: 0.5rem; color: #fbbf24; display: flex; align-items: center; gap: 0.4rem;">
          <span>⚠️</span> 臨床補打重要衛教提醒與限制：
        </h4>
        <ul style="padding-left: 1.25rem; margin: 0; color: #cbd5e1; font-size: 0.92rem; line-height: 1.6;">
          ${clinical_notes.map(note => `<li style="margin-bottom: 0.35rem;">${note}</li>`).join('')}
        </ul>
      </div>
    `;
  }

  container.innerHTML = `
    <div class="card catchup-result-card fade-in">
      <div class="catchup-card-header">
        <span class="catchup-tag">衛福部 ACIP 最小間隔指引</span>
        <h3>${vaccine_name} — 補打 ${next_dose_info}</h3>
      </div>

      <div class="catchup-date-banner">
        <div class="date-label">最早合法可補打日期</div>
        <div class="date-value">${earliest_date_display}</div>
        ${statusBadgeHtml}
      </div>

      <div class="catchup-rule-box">
        <div class="rule-title">📜 疾管署 ACIP 最小間隔 (Minimal Interval) 官方規定：</div>
        <div class="rule-text">${acip_rule_summary}</div>
      </div>

      ${notesHtml}
    </div>
  `;
}

// ----------------------------------------------------
// 分頁 4：出國旅遊醫學與留學疫苗速查 (Travel Medicine Advisory)
// ----------------------------------------------------
function setupTravelFormSubmit() {
  const form = document.getElementById('travel-form');
  if (!form) return;

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const destination = document.getElementById('travel-destination').value;
    const purpose = document.getElementById('travel-purpose').value;

    try {
      const res = await invoke('get_travel_advisory', { destination, purpose });
      displayTravelAdvisoryResult(res);
    } catch (err) {
      alert(`查詢失敗: ${err}`);
    }
  });
}

function displayTravelAdvisoryResult(data) {
  const container = document.getElementById('travel-result');
  if (!container) return;

  container.classList.remove('hidden');

  const { destination_name, purpose_name, mandatory_items, recommended_items, booster_items, travel_clinic_notes } = data;

  let mandatoryHtml = '';
  if (mandatory_items && mandatory_items.length > 0) {
    mandatoryHtml = `
      <div class="travel-section mandatory-section">
        <h4 class="travel-section-title mandatory-title">
          <span>🔴</span> 入境簽證 / 入學宿舍強制要求疫苗 (Mandatory Requirements)
        </h4>
        <div class="travel-cards-list">
          ${mandatory_items.map(item => `
            <div class="travel-item-card mandatory-card">
              <div class="travel-card-top">
                <span class="travel-badge mandatory-badge">${item.yellow_book_required ? '📜 需國際黃皮書 (Yellow Book)' : '📋 入學體檢表強制填報'}</span>
                <h5>${item.name}</h5>
              </div>
              <p class="travel-timing">⏱️ 建議時程：${item.timing_note}</p>
              <p class="travel-desc">${item.description}</p>
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }

  let recommendedHtml = '';
  if (recommended_items && recommended_items.length > 0) {
    recommendedHtml = `
      <div class="travel-section recommended-section">
        <h4 class="travel-section-title recommended-title">
          <span>🟡</span> 旅遊與留學強烈建議自費疫苗 (Highly Recommended)
        </h4>
        <div class="travel-cards-list">
          ${recommended_items.map(item => `
            <div class="travel-item-card recommended-card">
              <div class="travel-card-top">
                <span class="travel-badge recommended-badge">💰 自費強烈建議</span>
                <h5>${item.name}</h5>
              </div>
              <p class="travel-timing">⏱️ 建議時程：${item.timing_note}</p>
              <p class="travel-desc">${item.description}</p>
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }

  let boosterHtml = '';
  if (booster_items && booster_items.length > 0) {
    boosterHtml = `
      <div class="travel-section booster-section">
        <h4 class="travel-section-title booster-title">
          <span>🔵</span> 出國前常規追加疫苗 (Routine Booster)
        </h4>
        <div class="travel-cards-list">
          ${booster_items.map(item => `
            <div class="travel-item-card booster-card">
              <div class="travel-card-top">
                <span class="travel-badge booster-badge">💉 常規/定期追加</span>
                <h5>${item.name}</h5>
              </div>
              <p class="travel-timing">⏱️ 建議時程：${item.timing_note}</p>
              <p class="travel-desc">${item.description}</p>
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }

  let notesHtml = '';
  if (travel_clinic_notes && travel_clinic_notes.length > 0) {
    notesHtml = `
      <div class="travel-notes-box">
        <h4 style="margin-bottom: 0.5rem; color: #fbbf24; display: flex; align-items: center; gap: 0.4rem;">
          <span>🏥</span> 衛福部旅遊醫學門診特別提醒與藥物資訊：
        </h4>
        <ul style="padding-left: 1.25rem; margin: 0; color: #cbd5e1; font-size: 0.92rem; line-height: 1.6;">
          ${travel_clinic_notes.map(note => `<li style="margin-bottom: 0.35rem;">${note}</li>`).join('')}
        </ul>
      </div>
    `;
  }

  container.innerHTML = `
    <div class="card travel-result-card fade-in">
      <div class="travel-header-banner">
        <h3>${destination_name}</h3>
        <span class="travel-purpose-badge">${purpose_name}</span>
      </div>

      ${mandatoryHtml}
      ${recommendedHtml}
      ${boosterHtml}
      ${notesHtml}
    </div>
  `;
}

// ----------------------------------------------------
// 一鍵列印 / 匯出衛教建議單 (Print Report)
// 點擊後先彈出疫苗勾選 Modal，確認後再列印
// ----------------------------------------------------
function setupPrintButton() {
  const printBtn = document.getElementById('print-report-btn');
  if (!printBtn) return;

  printBtn.addEventListener('click', () => {
    if (!lastQueryData) {
      alert('請先選擇生日進行查詢，再行列印衛教建議單。');
      return;
    }
    openPrintSelectModal(lastQueryData);
  });
}

// 收集所有疫苗選項，開啟勾選 Modal
function openPrintSelectModal(data) {
  const { milestones } = data;

  // 建立疫苗列表：current + next 各自標記
  const vaccineItems = [];
  let currentMilestoneTitle = '';
  let nextMilestoneTitle = '';

  milestones.forEach(m => {
    if (m.status === 'Current' && currentMilestoneTitle === '') {
      currentMilestoneTitle = m.title;
      m.vaccines.forEach(v => {
        vaccineItems.push({ ...v, section: 'current', milestoneTitle: m.title });
      });
    } else if (m.status === 'Next' && nextMilestoneTitle === '') {
      nextMilestoneTitle = m.title;
      m.vaccines.forEach(v => {
        vaccineItems.push({ ...v, section: 'next', milestoneTitle: m.title });
      });
    }
  });

  // 渲染勾選清單
  const listEl = document.getElementById('print-select-list');
  if (!listEl) return;

  let html = '';
  if (vaccineItems.filter(v => v.section === 'current').length > 0) {
    html += `<div class="print-select-section-label">📍 當前階段：${currentMilestoneTitle}</div>`;
    vaccineItems.filter(v => v.section === 'current').forEach((v, i) => {
      const id = `ps-current-${i}`;
      let catLabel = '公費常規';
      if (v.category === 'Subsidized') catLabel = '縣市補助';
      else if (v.category === 'SelfPaid') catLabel = '自費建議';
      html += `
        <label class="print-select-item" for="${id}">
          <input type="checkbox" id="${id}" class="ps-checkbox" checked data-section="current" data-idx="${i}">
          <span class="ps-vaccine-name">${v.name}</span>
          <span class="ps-cat-badge ps-cat-${v.category?.toLowerCase() || 'routine'}">${catLabel}</span>
          <span class="ps-dose-info">${v.dose_info}</span>
        </label>`;
    });
  }

  if (vaccineItems.filter(v => v.section === 'next').length > 0) {
    html += `<div class="print-select-section-label" style="margin-top:0.5rem;">⏳ 下一階段：${nextMilestoneTitle}</div>`;
    vaccineItems.filter(v => v.section === 'next').forEach((v, i) => {
      const id = `ps-next-${i}`;
      let catLabel = '公費常規';
      if (v.category === 'Subsidized') catLabel = '縣市補助';
      else if (v.category === 'SelfPaid') catLabel = '自費建議';
      html += `
        <label class="print-select-item" for="${id}">
          <input type="checkbox" id="${id}" class="ps-checkbox" checked data-section="next" data-idx="${i}">
          <span class="ps-vaccine-name">${v.name}</span>
          <span class="ps-cat-badge ps-cat-${v.category?.toLowerCase() || 'routine'}">${catLabel}</span>
          <span class="ps-dose-info">${v.dose_info}</span>
        </label>`;
    });
  }

  listEl.innerHTML = html || '<div style="padding:1rem;color:#718096;">無疫苗項目可列印。</div>';

  // 更新計數
  const updateCount = () => {
    const total = document.querySelectorAll('.ps-checkbox:checked').length;
    const countEl = document.getElementById('print-select-count');
    if (countEl) countEl.textContent = `已勾選 ${total} 項`;
  };
  updateCount();
  listEl.querySelectorAll('.ps-checkbox').forEach(cb => cb.addEventListener('change', updateCount));

  // 全選 / 全部取消
  document.getElementById('print-select-all-btn').onclick = () => {
    listEl.querySelectorAll('.ps-checkbox').forEach(cb => { cb.checked = true; });
    updateCount();
  };
  document.getElementById('print-deselect-all-btn').onclick = () => {
    listEl.querySelectorAll('.ps-checkbox').forEach(cb => { cb.checked = false; });
    updateCount();
  };

  // 確認列印
  document.getElementById('print-select-confirm-btn').onclick = () => {
    const checkedCurrent = [...listEl.querySelectorAll('.ps-checkbox[data-section="current"]:checked')]
      .map(cb => vaccineItems.filter(v => v.section === 'current')[parseInt(cb.dataset.idx)]);
    const checkedNext = [...listEl.querySelectorAll('.ps-checkbox[data-section="next"]:checked')]
      .map(cb => vaccineItems.filter(v => v.section === 'next')[parseInt(cb.dataset.idx)]);

    closePrintSelectModal();
    prepareAndPrintReport(data, checkedCurrent, checkedNext);
  };

  // 關閉
  document.getElementById('print-select-modal-close').onclick = closePrintSelectModal;
  document.getElementById('print-select-cancel-btn').onclick = closePrintSelectModal;
  document.getElementById('print-select-modal').addEventListener('click', (e) => {
    if (e.target === document.getElementById('print-select-modal')) closePrintSelectModal();
  }, { once: true });

  document.getElementById('print-select-modal').classList.remove('hidden');
}

function closePrintSelectModal() {
  const modal = document.getElementById('print-select-modal');
  if (modal) modal.classList.add('hidden');
}

function prepareAndPrintReport(data, selectedCurrent, selectedNext) {
  const { age_display, child_age_detail, gender_display, location_display, current_visit_date, current_visit_milestone, next_visit_date, next_visit_milestone } = data;

  const printDate = document.getElementById('print-date');
  const printMeta = document.getElementById('print-meta-info');
  const printCurrentVisitInfo = document.getElementById('print-current-visit-info');
  const printNextVisitInfo = document.getElementById('print-next-visit-info');
  const currentTbody = document.getElementById('print-current-table-body');
  const nextTbody = document.getElementById('print-next-table-body');

  const now = new Date();
  if (printDate) {
    printDate.textContent = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
  }

  const ageText = child_age_detail || age_display;
  if (printMeta) {
    printMeta.textContent = `居住縣市：${location_display} ｜ 性別：${gender_display} ｜ 目前計算年齡：${ageText}`;
  }

  if (printCurrentVisitInfo) {
    printCurrentVisitInfo.textContent = `${current_visit_milestone || '當前階段'} (注射日期：${current_visit_date || '即日起符合'})`;
  }
  if (printNextVisitInfo) {
    printNextVisitInfo.textContent = `${next_visit_milestone || '定期保養追蹤'} (預估日期：${next_visit_date || '定期常規'})`;
  }

  // 填入勾選的疫苗
  currentTbody.innerHTML = '';
  nextTbody.innerHTML = '';

  if (selectedCurrent && selectedCurrent.length > 0) {
    selectedCurrent.forEach(v => {
      const tr = document.createElement('tr');
      let categoryLabel = '🏥 公費常規';
      if (v.category === 'Subsidized') categoryLabel = '🏛️ 縣市補助';
      else if (v.category === 'SelfPaid') categoryLabel = '💰 自費建議';
      tr.innerHTML = `
        <td style="font-weight:bold;">${v.name}</td>
        <td>${v.dose_info}</td>
        <td>${categoryLabel}</td>
        <td>${v.description}</td>
      `;
      currentTbody.appendChild(tr);
    });
  } else {
    currentTbody.innerHTML = '<tr><td colspan="4" style="text-align:center;color:#718096;">（本次無勾選當前階段項目）</td></tr>';
  }

  if (selectedNext && selectedNext.length > 0) {
    selectedNext.forEach(v => {
      const tr = document.createElement('tr');
      let categoryLabel = '🏥 公費常規';
      if (v.category === 'Subsidized') categoryLabel = '🏛️ 縣市補助';
      else if (v.category === 'SelfPaid') categoryLabel = '💰 自費建議';
      tr.innerHTML = `
        <td style="font-weight:bold;">${v.name}</td>
        <td>${v.dose_info}</td>
        <td>${categoryLabel}</td>
        <td>${v.description}</td>
      `;
      nextTbody.appendChild(tr);
    });
  } else {
    nextTbody.innerHTML = '<tr><td colspan="4" style="text-align:center;color:#718096;">（本次無勾選下一階段項目）</td></tr>';
  }

  // 填入同次接種組合與施打部位指南 (Co-administration Guide)
  const printCoadminContainer = document.getElementById('print-coadmin-container');
  const printCoadminList = document.getElementById('print-coadmin-list');
  if (printCoadminContainer && printCoadminList) {
    const currentMilestone = data.milestones ? data.milestones.find(m => m.status === 'Current') : null;
    const guides = (currentMilestone && currentMilestone.co_admin_guide && currentMilestone.co_admin_guide.length > 0)
      ? currentMilestone.co_admin_guide
      : [
          "不活化疫苗 (如五合一、肺炎鏈球菌、流感疫苗、B肝、A肝) 均可同次分開左右肢體部位接種，安全省時。",
          "口服減毒疫苗 (如輪狀病毒疫苗) 經腸道吸收，與肌肉針劑不互相干擾，可在針劑施打前後順利服用。",
          "鼻噴型流感疫苗 (AZ 能伏鼻 FluMist) 為活性減毒噴劑，免受扎針之苦；若與水痘/MMR等活性針劑未於同日完成需間隔至少 28 天。"
        ];

    printCoadminList.innerHTML = guides.map(g => `<li style="margin-bottom:0.25rem;">${g}</li>`).join('');
    printCoadminContainer.style.display = 'block';
  }

  window.print();
}


// ----------------------------------------------------
// 手機行事曆與 QR Code 提醒功能 (Google Calendar URL Generator)
// ----------------------------------------------------
function generateGoogleCalendarUrl(title, startDateStr, details) {
  const dateParts = startDateStr.split('-');
  let y = dateParts[0];
  let m = dateParts[1] ? dateParts[1].padStart(2, '0') : '01';
  let d = dateParts[2] ? dateParts[2].padStart(2, '0') : '01';

  const startFormatted = `${y}${m}${d}T090000`;
  const endFormatted = `${y}${m}${d}T100000`;

  const baseUrl = "https://www.google.com/calendar/render";
  const params = new URLSearchParams({
    action: "TEMPLATE",
    text: title,
    dates: `${startFormatted}/${endFormatted}`,
    details: `${details}\n\n提醒：請攜帶兒童預防接種紀錄黃卡與健保卡至診所就診。`,
    location: "預防接種醫療診所諮詢門診"
  });

  return `${baseUrl}?${params.toString()}`;
}

function openCalendarModal(title, dateDisplayStr, details) {
  const modal = document.getElementById('calendar-modal');
  const modalTitle = document.getElementById('cal-modal-title');
  const modalDate = document.getElementById('cal-modal-date');
  const directLink = document.getElementById('cal-direct-link');
  const copyBtn = document.getElementById('cal-copy-link-btn');

  const calUrl = generateGoogleCalendarUrl(title, dateDisplayStr, details);

  if (modalTitle) modalTitle.textContent = title;
  if (modalDate) modalDate.textContent = `預估建議日期：${dateDisplayStr}`;

  if (directLink) {
    directLink.href = calUrl;
    directLink.onclick = async (e) => {
      e.preventDefault();
      try {
        await invoke('launch_external_calendar_url', { url: calUrl });
      } catch (err) {
        if (window.__TAURI__?.opener?.openUrl) {
          window.__TAURI__.opener.openUrl(calUrl);
        } else {
          window.open(calUrl, '_blank');
        }
      }
    };
  }

  const qrContainer = document.getElementById('qrcode-container');
  if (qrContainer && window.QRCode) {
    qrContainer.innerHTML = ''; // 清除前一次生成的 QR Code，避免重疊
    new window.QRCode(qrContainer, {
      text: calUrl,
      width: 180,
      height: 180
    });
  }

  if (copyBtn) {
    copyBtn.onclick = () => {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(calUrl).then(() => {
          alert('已成功複製手機行事曆提醒連結！');
        }).catch(() => {
          alert('行事曆連結：\n' + calUrl);
        });
      } else {
        alert('行事曆連結：\n' + calUrl);
      }
    };
  }

  if (modal) modal.classList.remove('hidden');
}

function setupCalendarModalEvents() {
  const modal = document.getElementById('calendar-modal');
  const closeBtn = document.getElementById('calendar-modal-close');

  if (closeBtn) {
    closeBtn.addEventListener('click', () => modal.classList.add('hidden'));
  }

  if (modal) {
    modal.addEventListener('click', (e) => {
      if (e.target === modal) {
        modal.classList.add('hidden');
      }
    });
  }
}

// ----------------------------------------------------
// 分頁 5：0~18歲兒童與青少年生長曲線與 BMI 試算
// ----------------------------------------------------
function setupGrowthFormSubmit() {
  const ageSelect = document.getElementById('growth-age-years');
  if (!ageSelect) return;

  // 0~84 個月 (0~7歲前按月齡選項)
  const groupMonths = document.createElement('optgroup');
  groupMonths.label = '👶 0 ~ 6 歲（學齡前 - 依月齡）';
  for (let m = 0; m <= 83; m++) {
    const opt = document.createElement('option');
    opt.value = m;
    if (m === 0) {
      opt.textContent = '剛出生 (0 個月 / 新生兒)';
    } else if (m < 12) {
      opt.textContent = `滿 ${m} 個月大`;
    } else {
      const y = Math.floor(m / 12);
      const rem = m % 12;
      opt.textContent = rem === 0 ? `滿 ${y} 歲` : `滿 ${y} 歲 ${rem} 個月 (${m}月齡)`;
    }
    if (m === 6) opt.selected = true; // 預設 6 個月
    groupMonths.appendChild(opt);
  }
  ageSelect.appendChild(groupMonths);

  // 7~18 歲（學齡兒童與青少年 - 依足歲）
  const groupYears = document.createElement('optgroup');
  groupYears.label = '🎒 7 ~ 18 歲（學齡兒童與青少年 - 依足歲）';
  for (let y = 7; y <= 18; y++) {
    const opt = document.createElement('option');
    opt.value = y * 12; // 轉為月齡傳給 Rust
    opt.textContent = `滿 ${y} 歲 (${y} 歲學齡/青少年)`;
    groupYears.appendChild(opt);
  }
  ageSelect.appendChild(groupYears);

  // 根據選擇的年齡隱藏/顯示頭圍區塊
  ageSelect.addEventListener('change', () => {
    const months = parseInt(ageSelect.value) || 0;
    const headField = document.getElementById('growth-head-field');
    if (headField) {
      headField.style.display = months >= 84 ? 'none' : 'block';
    }
  });

  const form = document.getElementById('growth-form');
  if (!form) return;

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const gender = document.querySelector('input[name="growth-gender"]:checked').value;
    const ageMonths = parseInt(document.getElementById('growth-age-years').value) || 0;
    const height = parseFloat(document.getElementById('growth-height').value);
    const weight = parseFloat(document.getElementById('growth-weight').value);
    const headInput = document.getElementById('growth-head').value;
    const head = headInput ? parseFloat(headInput) : null;

    try {
      const response = await invoke('calculate_growth_percentile', {
        gender,
        ageMonths,
        height,
        weight,
        head,
      });
      displayGrowthResults(response);
    } catch (err) {
      alert(`生長曲線計算錯誤: ${err}`);
    }
  });
}

function displayGrowthResults(data) {
  const { age_display, gender_display, data_sources_citation, height_result, weight_result, bmi_result, head_result, overall_advice } = data;
  const container = document.getElementById('growth-results');
  if (!container) return;

  function renderMetricCard(metric) {
    if (!metric) return '';
    const badgeColor = metric.is_warning ? '#ef4444' : '#10b981';
    const progressPercent = Math.min(100, Math.max(3, metric.percentile_val));

    return `
      <div class="card growth-metric-card" style="padding: 1.25rem; margin-bottom: 1rem; border-left: 5px solid ${badgeColor};">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom: 0.75rem; flex-wrap:wrap; gap:0.5rem;">
          <h4 style="margin:0; font-size:1.1rem; color:#1e293b;">${metric.metric_name}：<span style="color:#0284c7; font-weight:700;">${metric.user_val} ${metric.unit}</span></h4>
          <span style="background:${metric.is_warning ? '#fee2e2' : '#dcfce7'}; color:${metric.is_warning ? '#991b1b' : '#166534'}; padding:0.35rem 0.75rem; border-radius:20px; font-weight:700; font-size:0.85rem;">
            ${metric.percentile_label}
          </span>
        </div>

        <!-- 條狀圖模擬百分位曲線 -->
        <div style="background:#e2e8f0; height:12px; border-radius:6px; overflow:hidden; position:relative; margin-bottom: 0.6rem;">
          <div style="width: ${progressPercent}%; background: linear-gradient(90deg, #38bdf8, #0284c7); height:100%; border-radius:6px;"></div>
        </div>

        <div style="display:flex; justify-content:space-between; font-size:0.75rem; color:#64748b;">
          <span>3% (${metric.p3.toFixed(1)}${metric.unit})</span>
          <span>50% (${metric.p50.toFixed(1)}${metric.unit})</span>
          <span>97% (${metric.p97.toFixed(1)}${metric.unit})</span>
        </div>

        <div style="margin-top:0.6rem; font-size:0.88rem; color:#475569;">
          📌 評估結論：<strong>${metric.status_summary}</strong>
        </div>
      </div>
    `;
  }

  container.innerHTML = `
    <div class="card fade-in" style="background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 100%); border: 1px solid #bae6fd; margin-bottom: 1.25rem;">
      <div style="display:flex; justify-content:space-between; align-items:flex-start; margin-bottom:0.4rem;">
        <h3 style="color:#0369a1; margin:0; font-size:1.25rem;">📊 生長發育與 BMI 評估報告</h3>
        <button onclick="printGrowthReport()" class="print-btn hide-on-print" style="font-size:0.85rem; padding:0.4rem 0.8rem; flex-shrink:0;">🖨️ 列印報告</button>
      </div>
      <div style="font-size:0.92rem; color:#0c4a6e; margin-bottom:0.5rem;">
        性別：<strong>${gender_display}</strong> ｜ 年齡層：<strong>${age_display}</strong>
      </div>
      <div style="font-size:0.78rem; color:#475569; border-top:1px dashed #bae6fd; padding-top:0.5rem; margin-top:0.5rem;">
        🏛️ <strong>本評估報告採用之國健署權威資料來源：</strong>
        <ul style="margin:0.2rem 0 0 1.2rem; padding:0;">
          ${data_sources_citation.map(src => `<li style="margin-bottom:2px;">${src}</li>`).join('')}
        </ul>
      </div>
    </div>

    ${renderMetricCard(height_result)}
    ${weight_result ? renderMetricCard(weight_result) : ''}
    ${bmi_result ? renderMetricCard(bmi_result) : ''}
    ${head_result ? renderMetricCard(head_result) : ''}

    <div class="card advice-card" style="background:#fffbeb; border:1px solid #fde68a; padding:1.25rem;">
      <h4 style="color:#b45309; margin-bottom:0.6rem; display:flex; align-items:center; gap:0.4rem;">
        <span>🩺</span> 國民健康署衛教建議與健康指標：
      </h4>
      <ul style="padding-left:1.25rem; margin:0; color:#78350f; font-size:0.88rem; line-height:1.6;">
        ${overall_advice.map(adv => `<li style="margin-bottom:0.35rem;">${adv}</li>`).join('')}
      </ul>
    </div>
  `;

  container.classList.remove('hidden');
  container.scrollIntoView({ behavior: 'smooth', block: 'start' });
}



// ==========================================
// 系統登入與權限控制
// ==========================================
function setupLoginSystem() {
  const loginOverlay = document.getElementById('login-overlay');
  const mainApp = document.getElementById('main-app-container');
  const loginForm = document.getElementById('login-form');
  const errorMsg = document.getElementById('login-error');

  // 檢查是否已經登入過 (存在 sessionStorage)
  if (sessionStorage.getItem('wuent_auth') === 'granted') {
    loginOverlay.classList.add('hidden');
    mainApp.classList.remove('hidden');
    return; // 已經登入，直接顯示主程式
  }

  // 攔截登入表單送出
  if (loginForm) {
    loginForm.addEventListener('submit', (e) => {
      e.preventDefault();
      const accountInput = document.getElementById('login-account').value.trim();
      const passwordInput = document.getElementById('login-password').value.trim();

      // 檢查帳號密碼
      if (accountInput === 'wuent' && passwordInput === '033787876') {
        // 登入成功
        sessionStorage.setItem('wuent_auth', 'granted');
        errorMsg.classList.add('hidden');
        
        // 隱藏登入畫面並顯示主畫面
        loginOverlay.classList.add('hidden');
        mainApp.classList.remove('hidden');
        
        // 如果是剛登入成功，可能需要觸發一次 Resize 讓一些排版(如日曆)重整
        setTimeout(() => window.dispatchEvent(new Event('resize')), 100);
      } else {
        // 登入失敗
        errorMsg.classList.remove('hidden');
        
        // 震動動畫提示錯誤
        const card = document.querySelector('.login-card');
        card.style.animation = 'none';
        card.offsetHeight; // trigger reflow
        card.style.animation = 'shake 0.4s ease';
      }
    });
  }
}


// Setup logout button
const logoutBtn = document.getElementById('logout-btn');
if (logoutBtn) {
  logoutBtn.addEventListener('click', () => {
    if (confirm('確定要登出系統嗎？')) {
      sessionStorage.removeItem('wuent_auth');
      window.location.reload();
    }
  });
}

// ==========================================
// 生長曲線專用列印功能
// ==========================================
window.printGrowthReport = function() {
  document.body.classList.add('print-mode-growth');
  window.print();
  // 為了保險起見，設定一個 timeout 清除 (某些瀏覽器不會觸發 afterprint)
  setTimeout(() => {
    document.body.classList.remove('print-mode-growth');
  }, 1000);
};

window.addEventListener('afterprint', () => {
  document.body.classList.remove('print-mode-growth');
});
