function toggleStyle() {
    if (document.getElementById("radio_style_v").checked) {
        document.body.removeAttribute("horizontal")
        lboxes = document.querySelectorAll(".l-box")
        for (var i = 0; i < lboxes.length; i++) {
            lboxes[i].removeAttribute("horizontal")
        }
        rboxes = document.querySelectorAll(".r-box")
        for (var i = 0; i < rboxes.length; i++) {
            rboxes[i].removeAttribute("horizontal")
        }
    } else {
        document.body.setAttribute("horizontal", "1")
        lboxes = document.querySelectorAll(".l-box")
        for (var i = 0; i < lboxes.length; i++) {
            lboxes[i].setAttribute("horizontal", "1")
        }
        rboxes = document.querySelectorAll(".r-box")
        for (var i = 0; i < rboxes.length; i++) {
            rboxes[i].setAttribute("horizontal", "1")
        }
    }
}
