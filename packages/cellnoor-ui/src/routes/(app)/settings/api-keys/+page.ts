import { cellnoorClient, MUTABLE_DATA, unwrap } from '#lib/client.ts';

export async function load({ depends }) {
	depends(MUTABLE_DATA);

	const [apiKeys, botAccounts] = await Promise.all([
		cellnoorClient.GET('/api-keys').then(unwrap),
		cellnoorClient.GET('/bot-accounts').then(unwrap)
	]);

	const botAccountNames = new Map(botAccounts.map((s) => [s.id, s.description ?? s.id]));

	return { apiKeys, botAccountNames };
}
