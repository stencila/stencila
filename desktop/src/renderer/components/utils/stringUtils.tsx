export const capitalize = ([head, ...tail]: string): string => {
  return [head?.toUpperCase() ?? '', ...tail].join('');
};
