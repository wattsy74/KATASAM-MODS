# 🐝 KATASAM Firmware Switcher - Logo Instructions

## ✅ Updates Applied

### Branding Changes:
- ✅ App name: "KATASAM Firmware Switcher"
- ✅ Color scheme: #EFCF5F (yellow/gold)
- ✅ Button text: "Apply" (instead of "Flash")
- ✅ Logo placeholder ready

---

## 🐝 ADD YOUR BEE LOGO

### Where to Put It:
Save your bee logo image as:
```
/Users/martynwatts/Library/CloudStorage/OneDrive-Personal/Desktop/firmware-flasher-web/electron-app/bee-logo.png
```

### Image Specifications:
- **Size:** 60x60 pixels (or 120x120 for retina)
- **Format:** PNG with transparent background
- **Name:** `bee-logo.png` (exactly this name)

---

## 🎨 Current Design

```
┌─────────────────────────────────┐
│          🐝 (logo)              │
│  KATASAM Firmware Switcher    │
│                                 │
│  ┌──────────────────────────┐  │
│  │ ● Device Name            │  │
│  │ Current firmware         │  │
│  └──────────────────────────┘  │
│                                 │
│  SELECT FIRMWARE                │
│  ┌────┐ ┌────┐ ┌────┐         │
│  │ C1 │ │ C2 │ │ ST │         │
│  └────┘ └────┘ └────┘         │
│                                 │
│  ┌─────────────────────────┐   │
│  │        Apply            │   │
│  └─────────────────────────┘   │
└─────────────────────────────────┘
```

**Colors:**
- Background: #1a1a1a (dark gray)
- Container: #2a2a2a (medium gray)
- Text: #EFCF5F (yellow/gold)
- Buttons: #EFCF5F when selected
- Apply button: #EFCF5F background, #1a1a1a text

---

## 🚀 Test With Placeholder

If you don't have the logo yet, you can test with a placeholder:

```bash
# Create a simple placeholder (optional)
cd "/Users/martynwatts/Library/CloudStorage/OneDrive-Personal/Desktop/firmware-flasher-web/electron-app"

# Run the app
npm start
```

The app will work without the logo (it just won't display an image).

---

## 📦 When Logo is Ready

1. Save bee logo as `bee-logo.png` in the electron-app folder
2. The logo will automatically appear centered above the title
3. Displays at 60x60 pixels

---

## ✅ All Changes Made:

### Files Updated:
- ✅ `index-minimal.html` - New branding, colors, logo spot
- ✅ `renderer-minimal.js` - "Apply" button text
- ✅ `main.js` - Window title
- ✅ `package.json` - App name and product name

### Visual Changes:
- ✅ All text now #EFCF5F (yellow/gold)
- ✅ Selected firmware button highlights in yellow
- ✅ Apply button is yellow
- ✅ Progress bar is yellow
- ✅ Logo space added with 60x60 size
- ✅ Dark theme maintained

### Interaction:
- ✅ Single "Apply" button (not "Flash Santroller")
- ✅ Minimal clicks (select firmware → Apply)

---

## 🐝 Next Steps:

1. **Add your bee logo** → Save as `bee-logo.png`
2. **Test the app** → `npm start`
3. **Build if happy** → `npm run build:mac:arm64`

---

**Ready for your bee logo!** 🐝✨
