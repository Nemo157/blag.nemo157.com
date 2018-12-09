let me = document.currentScript
let playground = me.previousElementSibling
let linkContainer = playground.previousElementSibling
let link = linkContainer.children[0]
let snippet = linkContainer.previousElementSibling.children[0].children[0]
link.href = `https://play.rust-lang.org/?version=nightly&edition=2018&code=${encodeURIComponent(playground.text)}`
link.style.display = ''
snippet.style = 'position: relative;'
snippet.prepend(link)
