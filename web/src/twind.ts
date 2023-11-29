import { defineConfig } from "@twind/core";
import presetAutoprefix from "@twind/preset-autoprefix";
import presetTailwind from "@twind/preset-tailwind";
import presetTypography from "@twind/preset-typography";
import install from "@twind/with-web-components";

const config = defineConfig({
  presets: [presetAutoprefix(), presetTailwind(), presetTypography()],
});

export const installTwind = () => install(config);
