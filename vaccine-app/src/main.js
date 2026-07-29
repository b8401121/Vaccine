const { invoke } = window.__TAURI__.tauri;

document.addEventListener('DOMContentLoaded', () => {
  const monthSelect = document.getElementById('month');
  const daySelect = document.getElementById('day');
  
  // Populate months
  for (let i = 1; i <= 12; i++) {
    const opt = document.createElement('option');
    opt.value = i;
    opt.textContent = i;
    monthSelect.appendChild(opt);
  }

  // Populate days
  for (let i = 1; i <= 31; i++) {
    const opt = document.createElement('option');
    opt.value = i;
    opt.textContent = i;
    daySelect.appendChild(opt);
  }

  const calendarToggle = document.getElementById('calendar-toggle');
  const calendarLabel = document.getElementById('calendar-label');
  const yearInput = document.getElementById('year');

  calendarToggle.addEventListener('change', (e) => {
    if (e.target.checked) {
      calendarLabel.textContent = '目前使用：民國';
      yearInput.placeholder = '如: 80';
    } else {
      calendarLabel.textContent = '目前使用：西元';
      yearInput.placeholder = '如: 1990';
    }
  });

  const form = document.getElementById('dob-form');
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const year = parseInt(yearInput.value);
    const month = parseInt(monthSelect.value);
    const day = parseInt(daySelect.value);
    const isRoc = calendarToggle.checked;

    try {
      const response = await invoke('get_eligible_vaccines', { year, month, day, isRoc });
      displayVaccines(response);
    } catch (error) {
      alert(`錯誤: ${error}`);
    }
  });
});

function displayVaccines(data) {
  const { age_display, groups } = data;
  const resultsDiv = document.getElementById('results');
  const groupListDiv = document.getElementById('group-list');
  const ageBadge = document.getElementById('age-badge');

  ageBadge.textContent = `目前計算年齡：${age_display}`;
  groupListDiv.innerHTML = '';

  if (!groups || groups.length === 0) {
    groupListDiv.innerHTML = '<p class="empty" style="color: #94a3b8; grid-column: 1 / -1;">目前無特定建議施打的疫苗。</p>';
  } else {
    groups.forEach(g => {
      const card = document.createElement('div');
      card.className = 'group-card fade-in';

      const tagClass = g.category === 'Routine' ? 'routine' : 'high-risk';
      const tagText = g.category === 'Routine' ? '常規建議' : '高風險對象';

      let dosesHtml = '';
      g.doses.forEach(d => {
        let statusBadge = '';
        let statusClass = '';
        if (d.status === 'Past') {
          statusBadge = '✓ 歷史應完成';
          statusClass = 'past-pill';
        } else if (d.status === 'Current') {
          statusBadge = '📍 當前應接種';
          statusClass = 'current-pill';
        } else {
          statusBadge = '⏳ 未來預計';
          statusClass = 'next-pill';
        }

        dosesHtml += `
          <div class="dose-row ${statusClass}-row">
            <div class="dose-left">
              <span class="dose-title">💉 ${d.dose_info}</span>
              <span class="timing-badge">📅 ${d.timing_info}</span>
            </div>
            <span class="status-pill ${statusClass}">${statusBadge}</span>
          </div>
          <p class="dose-desc">${d.description}</p>
        `;
      });

      card.innerHTML = `
        <div class="card-header">
          <h3>${g.name}</h3>
          <span class="tag ${tagClass}">${tagText}</span>
        </div>
        <div class="doses-container">
          ${dosesHtml}
        </div>
        <div class="audience-icon" style="margin-top: 1rem;">
          ${g.audience === 'Children' ? '🧸 幼兒/兒童疫苗' : '🧑 成人/長者疫苗'}
        </div>
      `;
      groupListDiv.appendChild(card);
    });
  }

  resultsDiv.classList.remove('hidden');
}
