import levenshtein from 'js-levenshtein';

/**
 * Given an invalid input string and a list of valid options, returns a filtered
 * list of valid options sorted based on their similarity with the input.
 */
export function suggestionList(input: string, options: string[]): string[] {
  const optionsByDistance = new Map<string, number>();

  const threshold = Math.floor(input.length * 0.4) + 1;
  const inputLowerCase = input.toLowerCase();
  for (const option of options) {
    // Special casing so that if the only mismatch is in uppper/lower-case, then the
    // option is always shown.
    const distance = inputLowerCase === option.toLowerCase()
      ? 1
      : levenshtein(input, option);
    if (distance <= threshold) {
      optionsByDistance.set(option, distance);
    }
  }

  return [...optionsByDistance.keys()].sort((a, b) => {
    const distanceDiff = optionsByDistance.get(a)! - optionsByDistance.get(b)!;
    return distanceDiff !== 0 ? distanceDiff : a.localeCompare(b);
  });
}

const MAX_SUGGESTIONS = 5;

/**
 * Given [ A, B, C ] return ' Did you mean A, B, or C?'.
 */
export function didYouMean(suggestions: readonly string[]): string {
  let message = ' Did you mean ';

  const quotedSuggestions = suggestions.map((x) => `"${x}"`);
  switch (suggestions.length) {
    case 0:
      return '';
    case 1:
      return message + quotedSuggestions[0] + '?';
    case 2:
      return message + quotedSuggestions[0] + ' or ' + quotedSuggestions[1] + '?';
  }

  const selected = quotedSuggestions.slice(0, MAX_SUGGESTIONS);
  const lastItem = selected.pop();
  return message + selected.join(', ') + ', or ' + lastItem + '?';
}

