// a script that gets all the wordLists for codebook and writes them to a file
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
const languages = ["rs", "py", "java", "html", "css", "go", "js"];

const wordListsPath = path.join(__dirname, "..", "word_lists");
// ensure the folder exists
fs.mkdirSync(wordListsPath, { recursive: true });
// map of language to wordList
// key: language in data repo, value: name in queries.rs

const fetch = async (url: string) => {
  const response = await globalThis.fetch(url);
  return response.json();
};

const getWordList = async (language: string) => {
  const url = `https://raw.githubusercontent.com/blopker/common-words/master/web/static/data/${language}/index.json`;
  const data = (await fetch(url)) as WordSummary[];
  const words = data
    .flatMap((d) => {
      return cleanWord(d.text.toLowerCase());
    })
    .filter((d) => d.length > 1);
  const set = new Set(words); // De-dupe
  return Array.from(set).toSorted();
};

const writeWordList = async (language: string) => {
  const words = await getWordList(language);
  const wordListPath = path.join(wordListsPath, `${language}.txt`);
  fs.writeFileSync(wordListPath, words.join("\n"));
};

const cleanWord = (word: string) => {
  // split on underscore, strip numbers and special characters
  const words = word.split(/[_\W]/);
  for (let i = 0; i < words.length; i++) {
    words[i] = words[i].replace(/[^a-zA-Z]/g, "");
  }
  return words;
};

const main = async () => {
  for (const language of languages) {
    await writeWordList(language);
  }
};

main();
