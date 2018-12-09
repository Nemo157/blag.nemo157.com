/* export onSwitcherClick */

if (!navigator.userAgent.match(/(iPad|iPhone|iPod)/)) {
  // Workaround buggy mobile safari hover handling
  document.body.classList.add('support-hover')
}

var storage = window.localStorage || { getItem: function () {}, setItem: function () {} }

var clamp = function (val) {
  return val && Math.max(Math.min(val, 26), 10)
}

var shade = storage.getItem('shade') || 'dark'
var size = clamp(parseInt(storage.getItem('size'), 10)) || 18

document.body.classList.remove('dark')
document.body.classList.remove('light')
document.body.classList.add(shade)
document.body.style.fontSize = size + 'px'

var switcher = document.getElementById('switcher')
switcher.style.display = ''

window.onInvertClick = function () {
  shade = shade === 'dark' ? 'light' : 'dark'
  storage.setItem('shade', shade)
  document.body.classList.toggle('dark')
  document.body.classList.toggle('light')
}

window.onPlusClick = function () {
  size = clamp(size + 1)
  storage.setItem('size', size)
  document.body.style.fontSize = size + 'px'
}

window.onMinusClick = function () {
  size = clamp(size - 1)
  storage.setItem('size', size)
  document.body.style.fontSize = size + 'px'
}
