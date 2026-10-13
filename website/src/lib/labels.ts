import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { FluentBundle, FluentResource, type FluentVariable } from '@fluent/bundle';

/** The app's strings, read from crates/tp-i18n/locales/<lang>/*.ftl. */
const LOCALES = join(process.cwd(), '..', 'crates', 'tp-i18n', 'locales');

const bundles = new Map<string, FluentBundle>();

function bundle(language: string): FluentBundle {
	let b = bundles.get(language);
	if (!b) {
		b = new FluentBundle(language, { useIsolating: false });
		const dir = join(LOCALES, language);
		for (const file of readdirSync(dir).filter((f) => f.endsWith('.ftl')).sort()) {
			const errors = b.addResource(new FluentResource(readFileSync(join(dir, file), 'utf8')));
			if (errors.length) throw new Error(`${dir}/${file}: ${errors.join(', ')}`);
		}
		bundles.set(language, b);
	}
	return b;
}

/**
 * The text the app shows for the message `id` in `language`. An unknown id
 * or a missing argument fails the build, naming `page`.
 */
export function label(
	language: string,
	id: string,
	page: string,
	args?: Record<string, FluentVariable>
): string {
	const b = bundle(language);
	const message = b.getMessage(id);
	if (!message?.value) {
		throw new Error(`Unknown interface label "${id}" (${language}) on page ${page}.`);
	}
	const errors: Error[] = [];
	const text = b.formatPattern(message.value, args ?? null, errors);
	if (errors.length) {
		throw new Error(`Interface label "${id}" (${language}) on page ${page}: ${errors.join(', ')}`);
	}
	return text;
}
