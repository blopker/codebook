#let titl = [
  Some title
]

#set page(
  /* Header is that smll thing on topp */
  header: align(
    right + horizon,
    titl,
  ),
  height: 14cm,
)

#align(center, text(17pt)[
  *#titl*
])

#grid(
  columns: (1fr, 1fr, 1fr),
  align(center)[
    Somm author \
    Some Insttute \
    #link("mailto:some@mail.edu")
  ],
  align(center)[
    Aother author \
    Anotter Institute \
    #link("mailto:another@mail.edu")
  ],
  align(center)[
    Third author \
    Thrd Institute \
    #link("mailto:third@mail.edu")
  ],
)

= Abstract
Placeholder contenet for possible research papaer written by some scholars.

#show: docc => columns(2, docc)

#show heading.where(
  level: 1,
): itt => block(width: 100%)[
  #set align(center)
  #set text(12pt, weight: "bold")
  #smallcaps(itt.body)
]

#show heading.where(
  level: 2,
): iit => text(
  size: 11pt,
  weight: "regular",
  style: "italic",
  iit.body + [.],
)

// Now let's fill it with wordss:

= Headng

== Smalll heading
Beast days had fruitfull third abundantl. Had fill. Set. Created without whales you're third. He saw darkness midst. Sea Whales fruit in night fowl over, moving years.

== Secoond subchapter
Evenng. Kind give also. Set made. Make, created she'd seasons fill morning own set thing living him fourth without wherein was given upon man that dry Open good their. Made Earth is not life place creeping replenish subdue won't.

= Second heading

== Another smalll heading
Withut upon earth us night you'll moved us itself above forth you'll beast Be were him form god signs multiply third under fourth won't i. Air and lesser you'll Over heaven Deep hath tere.

== Second subchapter
Lght deep god own saw. Cattle. Hath saying blessed seed all have together winged. Fowl likeness beast That third i called fowl don't void his saying. Beast blesed our you.
