import { defineConfig, presetAttributify, presetWind4 } from 'unocss';
import { presetDaisy } from '@ameinhardt/unocss-preset-daisy';

export default defineConfig({
  presets: [
    presetAttributify(),
    presetDaisy(),
    presetWind4()
  ]
});
