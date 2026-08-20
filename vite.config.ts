import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import Icons from "unplugin-icons/vite";

const reactSvgCompiler = {
  extension: "tsx",
  compiler(svg: string) {
    const jsx = svg
      .replace(/<svg /, "<svg {...props} ")
      .replace(/stroke-linecap/g, "strokeLinecap")
      .replace(/stroke-linejoin/g, "strokeLinejoin")
      .replace(/stroke-width/g, "strokeWidth")
      .replace(/fill-rule/g, "fillRule")
      .replace(/clip-rule/g, "clipRule")
      .replace(/class=/g, "className=");
    return `import type { SVGProps } from "react";
export default function Icon(props: SVGProps<SVGSVGElement>) {
  return (${jsx});
}`;
  },
} as const;

export default defineConfig({
  plugins: [react(), Icons({ compiler: reactSvgCompiler, autoInstall: false })],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
