import { chromium } from 'playwright';

interface CrawlResult {
    url: string;
    content: string;
    meta: any;
}

// The Ray worker calls this script
(async () => {
    const targetUrl = process.argv[2];
    if (!targetUrl) process.exit(1);

    const browser = await chromium.launch({ headless: true });
    // Stealth context setup
    const context = await browser.newContext({
        userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)...',
        viewport: { width: 1920, height: 1080 }
    });

    const page = await context.newPage();

    try {
        // Use CDP to mask automation
        const client = await context.newCDPSession(page);
        await client.send('Network.enable');
        
        // Navigate
        await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });

        // "Data Science" Heuristic in TS:
        // Calculate DOM text density to detect low-quality SPA filler
        const data = await page.evaluate(() => {
            const bodyText = document.body.innerText;
            const bodyHtml = document.body.innerHTML;
            return {
                text: bodyText,
                density: bodyText.length / bodyHtml.length
            };
        });

        if (data.density < 0.05) {
            console.error(JSON.stringify({ error: "Low density page" }));
        } else {
            const result: CrawlResult = {
                url: targetUrl,
                content: data.text,
                meta: { density: data.density }
            };
            console.log(JSON.stringify(result));
        }

    } catch (e) {
        console.error(JSON.stringify({ error: String(e) }));
    } finally {
        await browser.close();
    }
})();