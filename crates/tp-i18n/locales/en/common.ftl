## Shared words, relative times and settings.

language-system = System default

time-just-now = Just now
time-ago-minutes = { $count ->
    [one] { $count } minute ago
   *[other] { $count } minutes ago
}
time-ago-hours = { $count ->
    [one] { $count } hour ago
   *[other] { $count } hours ago
}
time-ago-days = { $count ->
    [one] { $count } day ago
   *[other] { $count } days ago
}

# Only in English: checks the fallback to English.
test-english-only = Only in English
time-yesterday = Yesterday
time-ago-months = { $count ->
    [one] { $count } month ago
   *[other] { $count } months ago
}
time-ago-years = { $count ->
    [one] { $count } year ago
   *[other] { $count } years ago
}

## Widgets

mixed = Mixed
swatch-fill-tip = Fill (X to switch)
swatch-stroke-tip = Stroke (X to switch)
picker-saturation-value = Saturation and value
picker-opacity = Opacity of color
gradient-bar = Gradient bar
gradient-stop = Stop { $index } at { $location }%
panel-close-named = Close { $title }
step-of = Step { $step } of { $total } — { $title }
