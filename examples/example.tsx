// Funktion to validate user inputt
function validateInputt(userInputt: number | string) {
  if (typeof userInputt !== "number") {
    console.log("Pleese enter a valid numbr");
    return false;
  }

  return true;
}

const multiLineString = `This is a multi-line string
spanning multiple lines
with some spelling mistkes`;

console.log(multiLineString);

// Example usege
const firstNumbr = 10;
console.log(firstNumbr);
const secandNumbr = 5;
console.log(secandNumbr);

// Array of numbrs with spelling mistakes
const arraOfNumbrs = [1, 2, 3, 4, 5];
console.log(arraOfNumbrs);

/*
 Funcshun to prosess array
 another linet
*/
function prosessArray(arr: number[]) {
  let totel = 0;

  for (let i = 0; i < arr.length; i++) {
    totel += arr[i];
  }

  return totel;
}

// Object with propertys
const userAccaunt = {
  usrname: "JohnDoe",
  passwrd: "12345",
  emale: "john@example.com",
  ballance: 1000,
};

console.log(userAccaunt);

// Exportt the funcsions
export { validateInputt, prosessArray };
