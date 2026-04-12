export function bytesToKB(bytes: number): string {
    return (bytes / 1024).toFixed(2) + " KB";
}

export function joinPaths(...paths: string[]): string {
    return '/' + paths.map(path => path.replace(/^\/|\/$/g, '')).filter(path => path !== '').join('/');
}