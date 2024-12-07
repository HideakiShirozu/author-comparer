import preact from '@preact/preset-vite'
import wyw from '@wyw-in-js/vite'
import { defineConfig } from 'vite'

// https://vitejs.dev/config/
export default defineConfig({
	plugins: [
		wyw({
			include: ['**/*.{ts,tsx}'],
			babelOptions: {
				presets: ['@babel/preset-typescript', '@babel/preset-react'],
			},
		}),
		preact({
			prerender: {
				enabled: true,
				renderTarget: '#app',
			},
		}),
	],
})
