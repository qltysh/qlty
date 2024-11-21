function f0(numbers): [number, number] {
  let mean = numbers.reduce((acc, num) => acc + num, 0) / numbers.length;

  let sortedNumbers = numbers.sort();
  let median;

  if (sortedNumbers.length % 2 === 0) {
    median =
      (sortedNumbers[sortedNumbers.length / 2 - 1] +
        sortedNumbers[sortedNumbers.length / 2]) /
      2;
  } else {
    median = sortedNumbers[Math.floor(sortedNumbers.length / 2)];
  }

  return [mean, median];
}

function f1(numbers): [number, number] {
  let mean = numbers.reduce((acc, num) => acc + num, 0) / numbers.length;

  let sortedNumbers = numbers.sort();
  let median;

  if (sortedNumbers.length % 2 === 0) {
    median =
      (sortedNumbers[sortedNumbers.length / 2 - 1] +
        sortedNumbers[sortedNumbers.length / 2]) /
      2;
  } else {
    median = sortedNumbers[Math.floor(sortedNumbers.length / 2)];
  }

  return [mean, median];
}
