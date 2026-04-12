export interface DirectoryEntry {
  name: string;
  entryType: 'file' | 'directory';
  size: number;
}

export interface Thumbnail {
  relativeBasePath: string;
  absoluteBasePath: string;
  thumbnailName: string;
  positionX: number;
  width: number;
  height: number;
  originalName: string;
  fileSize: number;
  selected?: boolean;
}