function toggleStyle() {
  if (document.querySelector('#radio_style_v').checked) {
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

document.addEventListener("DOMContentLoaded", () => {
  const elems = document.querySelectorAll('.latex')
  for (const elem of elems) {
    renderMathInElement(elem)
  }
})
