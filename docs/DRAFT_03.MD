# DRAFT_03 — Round 3 Q&A

Rounds 1 & 2 folded into the spec. This is the short tail — 5 items.
Fill in **A:**, say "done". After this the spec should be ready for
your review + commit, then the implementation plan.

---

### Q1. Linux artifact set
Proposed: **AppImage** (portable, runs on Fedora + Arch) **+ `.rpm`**
for Fedora now; an Arch `PKGBUILD` added later.
- [ ] Yes, that set
- [ ] AppImage only, skip `.rpm`
- [x] AppImage + `.rpm` + `PKGBUILD` all from the start
- [ ] Other:

**A:** AppImage + `.rpm` + `PKGBUILD` all from the start

### Q2. Map Drive fallback (Windows)
You picked "open the Map network drive wizard". Windows gives no
supported way to pre-fill the share path in that wizard. When the path
can't be pre-filled, the button should:
- [x] Open the wizard empty + copy `share_path` to the clipboard so you paste it
- [ ] Skip the wizard, run `net use` directly (no dialog)
- [ ] Just open Explorer at `\\host` (no Map Drive button at all)
- [ ] Other:

**A:** Open the wizard empty + copy `share_path` to the clipboard so you paste it

### Q3. Window size
VSCode-like layout wants a little more than the 2021 289×222. Proposed:
open at **~360×280**, minimum **~320×240**, resizable above that.
- [x] Fine
- [ ] Different numbers:

**A:** Fine

### Q4. TypeScript?
You said vanilla/plain JS. Confirming: frontend is **plain JS, no
TypeScript**, JSDoc for type hints where useful.
- [x] Correct, plain JS
- [ ] Actually, use TypeScript

**A:** Correct, plain JS + jsdoc

### Q5. First deliverable
Is the first thing we build **M1 (Linux) on its own** — get it working
end to end, then start Android — or do you want **M1 + M2 (Linux +
Android) delivered together** as one push?

**A:** M1 then M2. Step by step.

---

**Notes:**
