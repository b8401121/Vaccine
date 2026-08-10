// Wasm is loaded lazily via dynamic import so that even if loading fails,
// the rest of the page (tabs, date selectors, etc.) still works.
let wasmModule = null;

async function loadWasm() {
  if (wasmModule) return wasmModule;
  try {
    const mod = await import('./wasm2/vaccine_core.js?t=' + Date.now());
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
      lastQueryData = null;

      window.scrollTo({ top: 0, behavior: 'smooth' });
    });
  }

  const jumpBtn = document.getElementById('jump-to-current');
  if (jumpBtn) {
    jumpBtn.addEventListener('click', scrollToCurrentNode);
  }
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

async function loadVaccineLibrary() {
  try {
    allVaccinesList = await invoke('get_all_vaccines');
    renderLibraryGrid(allVaccinesList);
  } catch (err) {
    console.error('載入疫苗圖鑑庫失敗:', err);
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
    if (currentFilter === 'Routine') {
      matchesCategory = item.category === 'Routine' || item.category === 'Both';
    } else if (currentFilter === 'SelfPaid') {
      matchesCategory = item.category === 'SelfPaid' || item.category === 'Both';
    } else if (currentFilter === 'child') {
      matchesCategory = item.target_audience.includes('兒童') || item.target_audience.includes('全齡');
    } else if (currentFilter === 'adult') {
      matchesCategory = item.target_audience.includes('成人') || item.target_audience.includes('長者') || item.target_audience.includes('全齡');
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
