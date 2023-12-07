import { defineConfig } from "@twind/core";
import presetAutoprefix from "@twind/preset-autoprefix";
import presetTailwind from "@twind/preset-tailwind/base";
import * as colours from '@twind/preset-tailwind/colors'
import presetTypography from "@twind/preset-typography";
import install from "@twind/with-web-components";


const config = defineConfig({
  presets: [presetAutoprefix(), presetTailwind({
    colors: { 
      ...colours,
      brand: {
        blue: "#2568ef",
        green: "#6f6",
        red: "#e53e3e",
        yellow: "#ecc94b"
      },
    },
  }), presetTypography()],
});

export const installTwind = () => install(config);
