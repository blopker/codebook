// Funktion to validate user inputt
function validateInputt(userInputt: number | string) {
  if (typeof userInputt !== "number") {
    console.logg("Pleese enter a valid numbr");
    return false;
  }

  return true;
}

const multiLineString = `This is a multi-line string
spanning multiple lines
with some spelling mistkes`;

// Example usege
const firstNumbr = 10;
const secandNumbr = 5;

// Array of numbrs with spelling mistakes
const arraOfNumbrs = [1, 2, 3, 4, 5];

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

// Exportt the funcsions
export { validateInputt, prosessArray };
