// a script that gets all the wordlists for codebook and writes them to a file
// data in https://github.com/blopker/common-words/tree/master/web/static/data
// Use `fetch` to get the data
// folder format: [programming_language]/index.json
// json input format:
// ---
// [
//   {
//     "fontSize": 80,
//     "fontFamily": "Abel, sans-serif",
//     "x": 429,
//     "y": 864,
//     "dx": 0,
//     "text": "summary"
//   },
// ]
// ---
//
// output list format:
// ---
// word1
// word2
// word3
// ---
// Put everything in lower case
//
// The script also keeps a list of the most common programming languages, by their fontSize, that is output to common.txt

import fs from "node:fs";
import path from "node:path";

interface WordSummary {
  fontSize: number;
  fontFamily: string;
  x: number;
  y: number;
  dx: number;
  text: string;
}

const wordlistsPath = path.join(__dirname, "..", "wordlists");
// ensure the folder exists
fs.mkdirSync(wordlistsPath, { recursive: true });
// map of language to wordlist
// key: language in data repo, value: name in queries.rs
const languages = {
  rs: "rust",
  py: "python",
  java: "java",
  html: "html",
  css: "css",
  go: "go",
};

const commonWords: WordSummary[] = [];

function addToCommonWords(data: WordSummary) {
  for (const word of commonWords) {
    if (word.text === data.text) {
      return;
    }
  }
  if (commonWords.length <= 1000) {
    commonWords.push(data);
    commonWords.sort((a, b) => a.fontSize - b.fontSize);
    return;
  }
  const lowestScore = commonWords.length === 0 ? 0 : commonWords[0].fontSize;
  if (data.fontSize > lowestScore) {
    commonWords.shift();
    commonWords.push(data);
    commonWords.sort((a, b) => a.fontSize - b.fontSize);
  }
}

const fetch = async (url: string) => {
  const response = await globalThis.fetch(url);
  return response.json();
};

const getWordlist = async (language: string) => {
  const url = `https://raw.githubusercontent.com/blopker/common-words/master/web/static/data/${language}/index.json`;
  let data = (await fetch(url)) as WordSummary[];
  data = data.map((d) => {
    return {
      ...d,
      text: d.text.toLowerCase(),
    };
  });
  for (const item of data) {
    addToCommonWords(item);
  }
  const words = data.map((item: WordSummary) => item.text);
  return words;
};

const writeWordlist = async (language: string) => {
  const words = await getWordlist(language);
  const wordlistPath = path.join(wordlistsPath, `${language}.txt`);
  fs.writeFileSync(wordlistPath, words.join("\n"));
};

const main = async () => {
  for (const [language, _] of Object.entries(languages)) {
    await writeWordlist(language);
  }
  const commonWordsPath = path.join(wordlistsPath, "common.txt");
  fs.writeFileSync(
    commonWordsPath,
    commonWords
      .map((item) => item.text)
      .toSorted()
      .join("\n"),
  );
};

main();
