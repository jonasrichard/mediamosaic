export function bytesToKB(bytes: number): string {
    return formatNumberWithCommas((bytes / 1024).toFixed(2)) + " KB";
}

export function formatNumberWithCommas(num: string): string {
    return num.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

export function joinPaths(...paths: string[]): string {
    return '/' + paths.map(path => path.replace(/^\/|\/$/g, '')).filter(path => path !== '').join('/');
}