function toggleLayout() {
  const layoutVertical = document.querySelector('#layout-vertical').checked
  if (layoutVertical) {
    document.body.removeAttribute('horizontal')
    const lboxes = document.querySelectorAll('.l-box')
    for (const box of lboxes) {
      box.removeAttribute('horizontal')
    }
    const rboxes = document.querySelectorAll('.r-box')
    for (const box of rboxes) {
      box.removeAttribute('horizontal')
    }
  } else {
    document.body.setAttribute('horizontal', '1')
    const lboxes = document.querySelectorAll('.l-box')
    for (const box of lboxes) {
      box.setAttribute('horizontal', '1')
    }
    const rboxes = document.querySelectorAll('.r-box')
    for (const box of rboxes) {
      box.setAttribute('horizontal', '1')
    }
  }
}

function toggleDiffOnly() {
  const diffOnly = document.querySelector('#diff-only-yes').checked
  const entries = document.querySelectorAll('details.collapse.entry')
  for (const entry of entries) {
    if (diffOnly && !entry.hasAttribute('data-mark-red')) {
      entry.removeAttribute('open')
    } else {
      entry.setAttribute('open', '')
    }
  }
}

document.addEventListener('DOMContentLoaded', () => {
  document.querySelectorAll('.latex').forEach(renderMathInElement)
})
