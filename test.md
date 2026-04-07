MODE: STRICT_UI_COMPILER

TASK:
Create this UI in HTML + CSS only.
Output only the minimal HTML and CSS required.
Do not include explanations.

RULES:

- Output only what is explicitly described
- Do not add demo sections, wrappers, headings, labels, or examples
- Do not add hover, focus, active, pressed, or disabled states unless specified
- Do not add transitions, transforms, shadows, outlines, or effects unless specified
- Do not substitute fonts beyond declared fallbacks
- Do not center, pad, margin, or position elements unless specified
- Do not invent accessibility styles or interaction polish
- Do not expand scope beyond the described components
- Preserve component inheritance and variant structure
- Prefer minimal code over illustrative code

SCENE:

- background: #F3F4F6
- no scene padding or centering unless required by described coordinates

COMPONENT CARD_BASE:
kind: panel
frame:
width: 320px
height: 180px
box-sizing: border-box
shape:
primitive: rounded-rect
corner-radius: 16px
paint:
fill-color: #FFFFFF
border:
width: 1px
alignment: inner
color: #E5E7EB
shadow: none

COMPONENT BADGE_BASE:
kind: badge
frame:
height: 24px
box-sizing: border-box
shape:
primitive: pill
corner-radius: 999px
content-layout:
horizontal-align: center
vertical-align: center
label-typography:
font-family: Inter, sans-serif
font-size: 12px
font-weight: 600
line-height: 12px
letter-spacing: 0px

VARIANT BADGE_SUCCESS extends BADGE_BASE:
paint:
fill-color: #ECFDF5
border:
width: 1px
alignment: inner
color: #A7F3D0
label:
text: "Active"
color: #065F46
horizontal-padding:
left: 10px
right: 10px

COMPONENT BUTTON_BASE:
kind: button
element: native-button
reset: true
frame:
width: 88px
height: 36px
box-sizing: border-box
shape:
primitive: rounded-rect
corner-radius: 10px
content-layout:
horizontal-align: center
vertical-align: center
label-typography:
font-family: Inter, sans-serif
font-size: 14px
font-weight: 600
line-height: 14px
letter-spacing: 0px
interaction:
cursor: pointer

VARIANT BUTTON_PRIMARY extends BUTTON_BASE:
paint:
fill-color: #111827
border:
width: 0px
shadow: none
label:
text: "Open"
color: #FFFFFF

VARIANT BUTTON_SECONDARY extends BUTTON_BASE:
paint:
fill-color: #FFFFFF
border:
width: 1px
alignment: inner
color: #D1D5DB
shadow: none
label:
text: "Later"
color: #111827

COMPONENT PROFILE_CARD extends CARD_BASE:
layout:
position: absolute
x: 24px
y: 20px
children: - AVATAR - NAME - EMAIL - BADGE_SUCCESS - BUTTON_PRIMARY - BUTTON_SECONDARY - STATUS_DOT - STATUS_TEXT

NODE AVATAR:
kind: shape
parent: PROFILE_CARD
frame:
x: 20px
y: 20px
width: 48px
height: 48px
box-sizing: border-box
shape:
primitive: circle
paint:
fill-color: #DBEAFE
border:
width: 0px
content:
text: "A"
color: #1D4ED8
font-family: Inter, sans-serif
font-size: 20px
font-weight: 700
line-height: 20px
letter-spacing: 0px
horizontal-align: center
vertical-align: center

NODE NAME:
kind: text
parent: PROFILE_CARD
frame:
x: 84px
y: 22px
width: 140px
height: 20px
text:
value: "Ava Martinez"
typography:
font-family: Inter, sans-serif
font-size: 16px
font-weight: 600
line-height: 20px
letter-spacing: 0px
color: #111827

NODE EMAIL:
kind: text
parent: PROFILE_CARD
frame:
x: 84px
y: 46px
width: 180px
height: 18px
text:
value: "ava@northstar.dev"
typography:
font-family: Inter, sans-serif
font-size: 13px
font-weight: 400
line-height: 18px
letter-spacing: 0px
color: #6B7280

INSTANCE BADGE_SUCCESS AS PROFILE_BADGE:
parent: PROFILE_CARD
frame:
x: 20px
y: 88px

NODE STATUS_DOT:
kind: shape
parent: PROFILE_CARD
frame:
x: 20px
y: 126px
width: 8px
height: 8px
shape:
primitive: circle
paint:
fill-color: #10B981
border:
width: 0px

NODE STATUS_TEXT:
kind: text
parent: PROFILE_CARD
frame:
x: 34px
y: 121px
width: 120px
height: 18px
text:
value: "Last sync 2 min ago"
typography:
font-family: Inter, sans-serif
font-size: 13px
font-weight: 400
line-height: 18px
letter-spacing: 0px
color: #6B7280

INSTANCE BUTTON_SECONDARY AS PROFILE_LATER:
parent: PROFILE_CARD
frame:
x: 116px
y: 124px

INSTANCE BUTTON_PRIMARY AS PROFILE_OPEN:
parent: PROFILE_CARD
frame:
x: 212px
y: 124px

OUTPUT_RULES:
preserve-inheritance: true
no-extra-styles: true
no-implicit-shadow: true
no-implicit-padding: true
no-implicit-font-substitution: true
use-border-box: true
no-unrequested-states: true

results: worked on all 3 models, very well
