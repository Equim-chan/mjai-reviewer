function toggleLayout() {
  const layoutVertical = document.querySelector('#layout-vertical').checked
  if (layoutVertical) {
    document.body.removeAttribute('horizontal')
    const lboxes = document.querySelectorAll('.l-box')
    for (let i = 0; i < lboxes.length; i++) {
      lboxes[i].removeAttribute('horizontal')
    }
    const rboxes = document.querySelectorAll('.r-box')
    for (let i = 0; i < rboxes.length; i++) {
      rboxes[i].removeAttribute('horizontal')
    }
  } else {
    document.body.setAttribute('horizontal', '1')
    const lboxes = document.querySelectorAll('.l-box')
    for (let i = 0; i < lboxes.length; i++) {
      lboxes[i].setAttribute('horizontal', '1')
    }
    const rboxes = document.querySelectorAll('.r-box')
    for (let i = 0; i < rboxes.length; i++) {
      rboxes[i].setAttribute('horizontal', '1')
    }
  }
}

function toggleDiffOnly() {
  const diffOnly = document.querySelector('#diff-only-yes').checked
  const turnList = document.querySelectorAll('details.collapse')
  for (const turn of turnList) {
    const turnInfo = turn.querySelector('.turn-info')
    if (turnInfo) {
      if (diffOnly && !turnInfo.querySelector('.order-loss')) {
        turn.removeAttribute('open')
      } else {
        turn.setAttribute('open', '')
      }
    }
  }
}

document.addEventListener('DOMContentLoaded', () => {
  const elems = document.querySelectorAll('.latex')
  for (const elem of elems) {
    renderMathInElement(elem)
  }
})
