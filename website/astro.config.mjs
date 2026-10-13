// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

/** The seven sections, in sidebar order: directory and label per language. */
const sections = [
	['getting-started', 'Getting started', 'Premiers pas', 'Primeros pasos', 'Erste Schritte'],
	['vehicles', 'Vehicles', 'Véhicules', 'Vehículos', 'Fahrzeuge'],
	['fleet', 'Project and fleet', 'Projet et flotte', 'Proyecto y flota', 'Projekt und Flotte'],
	['workshop', 'Workshop', 'Atelier', 'Taller', 'Werkstatt'],
	['brand', 'Brand', 'Marque', 'Marca', 'Marke'],
	['export', 'Export', 'Export', 'Exportar', 'Export'],
	['ai-assistant', 'AI assistant (MCP)', 'Assistant IA (MCP)', 'Asistente de IA (MCP)', 'KI-Assistent (MCP)'],
];

export default defineConfig({
	site: 'https://truckpaint.ace-dev.me',
	trailingSlash: 'always',
	redirects: { '/': '/en/' },
	integrations: [
		starlight({
			title: 'TruckPaint',
			customCss: ['./src/styles/tokens.css', './src/styles/theme.css'],
			defaultLocale: 'en',
			locales: {
				en: { label: 'English' },
				fr: { label: 'Français' },
				es: { label: 'Español' },
				de: { label: 'Deutsch' },
			},
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/acesyde/etsats2-paint' },
			],
			sidebar: sections.map(([directory, en, fr, es, de]) => ({
				label: en,
				translations: { fr, es, de },
				items: [{ autogenerate: { directory } }],
			})),
		}),
	],
});
