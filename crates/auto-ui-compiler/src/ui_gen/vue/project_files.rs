//! PLAN-014 T-04：工程脚手架责任（package.json/vite/tailwind/components.json/utils.ts）（自 ui_gen/vue.rs impl 块原样收编）。
//! 单 VueGenerator state、公开路径与字节输出零改——纯 impl 代码移动；
//! 方法可见性按跨模块调用需要提升（其余保持私有语义）。
use super::*;

impl VueGenerator {
    /// Generate components.json for shadcn-vue project setup
    pub fn generate_components_json() -> String {
        r#"{
  "$schema": "https://shadcn-vue.com/schema.json",
  "style": "default",
  "typescript": true,
  "tailwind": {
    "config": "tailwind.config.cjs",
    "css": "src/assets/index.css",
    "baseColor": "slate",
    "cssVariables": true
  },
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils"
  }
}"#
        .to_string()
    }

    /// Generate package.json for shadcn-vue project
    pub fn generate_package_json(project_name: &str) -> String {
        format!(
            r#"{{
  "name": "{}",
  "version": "0.0.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "vue": "^3.4.0",
    "vue-router": "^4.2.0",
    "@vueuse/core": "^10.7.0",
    "radix-vue": "^1.4.0",
    "reka-ui": "^2.0.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.2.0",
    "lucide-vue-next": "^0.312.0",
    "embla-carousel-vue": "^8.5.1",
    "vee-validate": "^4.15.1",
    "@vee-validate/zod": "^4.15.1",
    "zod": "^3.25.76",
    "prismjs": "^1.29.0"
  }},
  "devDependencies": {{
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0",
    "typescript": "^5.3.0",
    "vue-tsc": "^2.0.0",
    "tailwindcss": "^3.4.0",
    "tailwindcss-animate": "^1.0.7",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "@types/prismjs": "^1.26.0"
  }}
}}"#,
            project_name
        )
    }

    /// Generate vite.config.ts for shadcn-vue project
    pub fn generate_vite_config() -> String {
        r#"import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        entryFileNames: 'assets/index.js',
        chunkFileNames: 'assets/[name].js',
        assetFileNames: 'assets/[name].[ext]',
      },
    },
  },
})
"#
        .to_string()
    }

    /// Generate tailwind.config.js for shadcn-vue project
    pub fn generate_tailwind_config() -> String {
        r#"/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
	      borderRadius: {
	        lg: "var(--radius)",
	        md: "calc(var(--radius) - 2px)",
	        sm: "calc(var(--radius) - 4px)",
	      },
	      keyframes: {
	        "accordion-down": {
	          from: { height: 0 },
	          to: { height: "var(--radix-accordion-content-height)" },
	        },
	        "accordion-up": {
	          from: { height: "var(--radix-accordion-content-height)" },
	          to: { height: 0 },
	        },
	        "collapsible-down": {
	          from: { height: 0, opacity: 0 },
	          to: { height: "var(--radix-collapsible-content-height)", opacity: 1 },
	        },
	        "collapsible-up": {
	          from: { height: "var(--radix-collapsible-content-height)", opacity: 1 },
	          to: { height: 0, opacity: 0 },
	        },
	      },
	      animation: {
	        "accordion-down": "accordion-down 0.2s ease-out",
	        "accordion-up": "accordion-up 0.2s ease-out",
	        "collapsible-down": "collapsible-down 0.2s ease-out",
	        "collapsible-up": "collapsible-up 0.2s ease-out",
	      },
	      boxShadow: {
	        "card": "var(--card-shadow)",
	      },
    },
  },
  plugins: [require("tailwindcss-animate")],
}
"#
        .to_string()
    }

    /// Generate lib/utils.ts for shadcn-vue project
    pub fn generate_utils_ts() -> String {
        r#"import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
"#
        .to_string()
    }

}
