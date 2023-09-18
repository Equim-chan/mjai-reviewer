function toggleLayout() {
  const layout = document.querySelector('#panel input[name="layout"]:checked').value;
  if (layout === 'vertical') {
    document.body.removeAttribute('data-horizontal');
    const lboxes = document.querySelectorAll('.l-box');
    for (const box of lboxes) {
      box.removeAttribute('data-horizontal');
    }
    const rboxes = document.querySelectorAll('.r-box');
    for (const box of rboxes) {
      box.removeAttribute('data-horizontal');
    }
  } else {
    document.body.setAttribute('data-horizontal', '');
    const lboxes = document.querySelectorAll('.l-box');
    for (const box of lboxes) {
      box.setAttribute('data-horizontal', '');
    }
    const rboxes = document.querySelectorAll('.r-box');
    for (const box of rboxes) {
      box.setAttribute('data-horizontal', '');
    }
  }
}

function toggleExpand() {
  const expand = document.querySelector('#panel input[name="expand"]:checked').value;
  const entries = document.querySelectorAll('details.collapse.entry');
  for (const entry of entries) {
    switch (expand) {
      case 'all':
        entry.open = true;
        break;
      case 'diff-only':
        entry.open = entry.hasAttribute('data-mark-red');
        break;
      case 'none':
        entry.open = false;
        break;
    }
  }
}

function goToTop() {
  document.body.scrollTop = 0;
  document.documentElement.scrollTop = 0;
}

function scrollHandler() {
  const topButton = document.querySelector('#top-button');
  if (document.body.scrollTop > 20 || document.documentElement.scrollTop > 20) {
    topButton.classList.add('show');
  } else {
    topButton.classList.remove('show');
  }
}

document.addEventListener('scroll', scrollHandler);

document.addEventListener('DOMContentLoaded', () => {
  scrollHandler();
  document.querySelectorAll('.latex').forEach(renderMathInElement);
});
