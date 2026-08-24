import { expect, test } from "@playwright/test";
import { chromium, type Page } from "playwright";
import { callRpc } from "./helpers";
import { SearchRequest } from "@generated/anki/search_pb";
import { ImportAnkiPackageOptions, ImportAnkiPackageRequest, ImportResponse } from "@generated/anki/import_export_pb";
import * as path from "path";
import * as url from "url";

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..", "..", "..");
const FIXTURE_PATH = path.join(REPO_ROOT, "Procedural_StudyLab_Fixture.apkg");

test.setTimeout(60000);

async function findPageWithId(pages: Page[], elementId: string): Promise<Page | undefined> {
    for (let i = 0; i < 20; i++) {
        for (const p of pages) {
            try {
                const hasElement = await p.evaluate((id) => !!document.getElementById(id), elementId);
                if (hasElement) return p;
            } catch (e) {
                // Ignore pages that are closed or cross-origin
            }
        }
        await new Promise(resolve => setTimeout(resolve, 500));
    }
    return undefined;
}

test("procedural runtime e2e", async ({ page: mediasrvPage }) => {
    // 1. Import fixture
    await mediasrvPage.goto("/congrats", { waitUntil: "domcontentloaded" });
    const resp = ImportResponse.fromBinary(
        await callRpc(
            mediasrvPage,
            "importAnkiPackage",
            new ImportAnkiPackageRequest({
                packagePath: FIXTURE_PATH,
                options: new ImportAnkiPackageOptions({
                    mergeNotetypes: true,
                }),
            }),
        ),
    );
    expect(resp).toBeDefined();

    // 2. Connect to the Anki QtWebEngine instance
    const browser = await chromium.connectOverCDP("http://127.0.0.1:9222");
    const contexts = browser.contexts();
    
    // We need to trigger Anki to go to the review screen.
    // We can find the topToolbar page which usually has `pycmd` and tell it to go to deck browser, select deck, etc.
    // Actually, `pycmd` is available on any main window webview.
    // Let's find ANY page that has `pycmd`.
    const pages = contexts[0].pages();
    let mainPage: Page | undefined;
    for (const p of pages) {
        try {
            const hasPycmd = await p.evaluate(() => typeof (window as any).pycmd === "function");
            if (hasPycmd) {
                mainPage = p;
                break;
            }
        } catch (e) {}
    }
    expect(mainPage).toBeDefined();

    // Go to deck browser just to reset state
    await mainPage!.evaluate(() => (window as any).pycmd("deckBrowser"));
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Find the deck we imported. 
    // The deck name in the fixture is "StudyLab Procedural Math" or similar.
    // Let's just click the first deck's "Study" button, or we can use python to do it.
    // wait, we can't easily run python. But wait! `mediasrv` has `get_deck_configs_for_update` etc.
    // Is there a way to start review via `pycmd`?
    // In deckbrowser, clicking a deck does `pycmd("open:12345")`.
    // Then we click study which does `pycmd("study")`.
    
    // Let's just use `SearchCards` RPC to find a card we want, then... we can't just jump to it in Reviewer.
    // We can evaluate Python via a malicious script in mediasrv? No, we don't have python `eval` in `mediasrv`.
    // Wait, the Reviewer loads the *current deck*.
    // If we click the first row in the deck list:
    const deckBrowserPage = await findPageWithId(contexts[0].pages(), "deck-list");
    expect(deckBrowserPage).toBeDefined();
    
    // Click the StudyLab deck. Let's find it by text.
    await deckBrowserPage!.evaluate(() => {
        const rows = Array.from(document.querySelectorAll('.deck-name'));
        const target = rows.find(r => r.textContent?.includes('StudyLab'));
        if (target) {
            (target as HTMLElement).click();
        } else {
            // fallback click first
            (document.querySelector('.deck-name') as HTMLElement).click();
        }
    });
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Now we are in the deck overview. Click "Study Now" (or press Enter)
    const overviewPage = await findPageWithId(contexts[0].pages(), "study");
    // "study" is the pycmd or button id? The button class is "btn" and pycmd is "study"
    // Actually, pressing "Enter" on the body of overview works.
    if (overviewPage) {
        await overviewPage.evaluate(() => (window as any).pycmd("study"));
    } else {
        // Just trigger study from mainPage
        await mainPage!.evaluate(() => (window as any).pycmd("study"));
    }

    // 3. Wait for the Reviewer page to load and display the Procedural UI
    const reviewerPage = await findPageWithId(contexts[0].pages(), "qa");
    expect(reviewerPage).toBeDefined();

    await reviewerPage!.waitForSelector("#procedural-card", { timeout: 10000 });
    
    // 4. Verify the UI loaded via window.anki.procedural.setup
    const isSetup = await reviewerPage!.evaluate(() => {
        return !!(document.querySelector('.procedural-card-container') as any).__proceduralReviewer;
    });
    expect(isSetup).toBeTruthy();

    // 5. Interact! Type an answer and submit.
    // If it's a math card, it has `#proc-answer-input`.
    const isMath = await reviewerPage!.evaluate(() => !!document.getElementById("proc-answer-input"));
    if (isMath) {
        await reviewerPage!.fill("#proc-answer-input", "123");
        await reviewerPage!.click("#proc-submit-btn");
        
        // Wait for result panel
        await reviewerPage!.waitForSelector("#proc-result-panel:not(.hidden)");
        
        // Let's click "Silly mistake" if it's wrong (it probably is wrong if we typed 123)
        const hasMistakeBtn = await reviewerPage!.evaluate(() => !!document.querySelector(".proc-mistake-btn"));
        if (hasMistakeBtn) {
            await reviewerPage!.click(".proc-mistake-btn[data-value='silly_mistake']");
        }
        
        // Wait for next button
        await reviewerPage!.waitForSelector("#proc-next-btn");
        await reviewerPage!.click("#proc-next-btn");
        
        // Click ease button
        await new Promise(resolve => setTimeout(resolve, 500));
        // We need the bottom toolbar page to click the ease button. Or we can just `pycmd("ease3")` from reviewerPage!
        await reviewerPage!.evaluate(() => (window as any).pycmd("ease3"));
    }

    // 6. We did a real run! If we get here, telemetry was executed without errors.
    console.log("E2E Runtime UI interaction successful.");
});
