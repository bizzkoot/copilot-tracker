import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const mainPath = path.resolve('src-tauri/src/main.rs');
const mainSource = fs.readFileSync(mainPath, 'utf8');

test('startup rebuild happens after widget restore block', () => {
  const initIndex = mainSource.indexOf('// Initialize widget state from settings');
  const rebuildIndex = mainSource.indexOf(
    '// Update tray menu after widget restoration to keep label in sync',
  );

  assert.notEqual(initIndex, -1, 'missing widget restore section marker');
  assert.notEqual(rebuildIndex, -1, 'missing post-restore tray rebuild marker');
  assert.ok(
    rebuildIndex > initIndex,
    'tray rebuild must run after widget restoration, not before',
  );
});

test('startup visible-widget path triggers immediate tray rebuild after show', () => {
  const visibleShowRebuildPattern =
    /if widget_visible \{[\s\S]*?show_widget_without_focus\(&widget, widget_pinned\)[\s\S]*?rebuild_tray_menu\(app\.handle\(\), latest\.as_ref\(\)\)/;

  assert.match(
    mainSource,
    visibleShowRebuildPattern,
    'expected immediate tray rebuild in widget_visible startup path',
  );
});

test('show_widget_without_focus uses parameterized pinned state', () => {
  assert.ok(
    mainSource.includes('fn show_widget_without_focus(') &&
      mainSource.includes('always_on_top: bool'),
    'show_widget_without_focus should accept always_on_top parameter',
  );

  assert.doesNotMatch(
    mainSource,
    /set_always_on_top\(true\)/,
    'hardcoded always_on_top=true should not exist in startup show helper',
  );
});
