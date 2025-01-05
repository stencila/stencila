#!/bin/bash

src="../../../../node_modules/@shoelace-style/shoelace/cdn/assets/icons"

icons=(
    activity
    archive
    arrow-bar-up
    arrow-clockwise
    arrow-left-square
    arrow-repeat
    arrow-right
    arrow-right-square
    arrow-up-circle
    arrow-up-circle-fill
    asterisk
    at
    ban
    box
    box-arrow-in-left
    box-arrow-up-right
    braces
    braces-asterisk
    brush
    building
    camera-video
    card-text
    chat-right-dots
    chat-right-text
    chat-square-text
    check
    check-circle
    check-circle-fill
    chevron-down
    chevron-left
    chevron-right
    circle
    clock
    code
    code-slash
    cone-striped
    crosshair
    currency-dollar
    dash-circle
    exclamation-circle
    exclamation-triangle
    eye
    eye-slash
    fast-forward-circle
    feather
    file-play
    file-plus
    filetype-raw
    fullscreen
    gear
    hand-thumbs-down
    hand-thumbs-down-fill
    hand-thumbs-up
    hand-thumbs-up-fill
    hash
    hr
    image
    image-alt
    info-circle
    lightbulb
    lightning
    lock
    paperclip
    person
    play
    play-circle
    plus-circle
    postage
    question-circle
    quote
    repeat
    shield-check
    skip-end
    skip-start
    slash-circle
    sliders
    speedometer
    square
    star-fill
    stopwatch
    table
    terminal
    thermometer
    toggle-off
    volume-up
    x-circle
)

for icon in "${icons[@]}"; do
    if [ -e "$src/$icon.svg" ]; then
        cp "$src/$icon.svg" .
        # Remove width and height to avoid viewBox stripping (see note in icon.ts)
        sed -i 's/width="16" height="16" //g' "$icon.svg"
        # Remove the namespace just to save some space
        sed -i 's!xmlns="http://www.w3.org/2000/svg"!!g' "$icon.svg"
    else
        echo "Icon $icon.svg not found in $src."
    fi
done

cp "$src/list.svg" bars.svg

cp "$src/LICENSE" .
