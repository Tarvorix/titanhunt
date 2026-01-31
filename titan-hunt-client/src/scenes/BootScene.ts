// Boot scene - loads game assets

import Phaser from 'phaser';
import { AtlasManifest } from '../types';

export class BootScene extends Phaser.Scene {
  private manifest: AtlasManifest | null = null;

  constructor() {
    super({ key: 'BootScene' });
  }

  preload(): void {
    // Create loading progress bar
    const width = this.cameras.main.width;
    const height = this.cameras.main.height;

    const progressBar = this.add.graphics();
    const progressBox = this.add.graphics();
    progressBox.fillStyle(0x222222, 0.8);
    progressBox.fillRect(width / 2 - 160, height / 2 - 25, 320, 50);

    const loadingText = this.add.text(width / 2, height / 2 - 50, 'Loading...', {
      font: '20px monospace',
      color: '#ffffff',
    });
    loadingText.setOrigin(0.5, 0.5);

    const percentText = this.add.text(width / 2, height / 2, '0%', {
      font: '18px monospace',
      color: '#ffffff',
    });
    percentText.setOrigin(0.5, 0.5);

    this.load.on('progress', (value: number) => {
      percentText.setText(`${Math.floor(value * 100)}%`);
      progressBar.clear();
      progressBar.fillStyle(0xffffff, 1);
      progressBar.fillRect(width / 2 - 150, height / 2 - 15, 300 * value, 30);
    });

    this.load.on('complete', () => {
      progressBar.destroy();
      progressBox.destroy();
      loadingText.destroy();
      percentText.destroy();
    });

    // Load the atlas manifest
    this.load.json('manifest', 'atlases/manifest.json');

    // Load all sprite atlases from the atlases folder
    const atlases = [
      'shadowsword',
      'shadowsword2',
      'shadowsword3',
      'Reaver_Titan',
      'Warlord_Titan',
      'krieg',
    ];

    for (const atlas of atlases) {
      this.load.atlas(
        atlas,
        `atlases/${atlas}.png`,
        `atlases/${atlas}.json`
      );
    }
  }

  create(): void {
    // Get the loaded manifest
    this.manifest = this.cache.json.get('manifest') as AtlasManifest;

    if (this.manifest) {
      console.log('Loaded atlas manifest:', this.manifest);
      console.log(`Found ${this.manifest.sheets.length} sprite sheets`);
    }

    // Transition to the battle scene
    this.scene.start('BattleScene', { manifest: this.manifest });
  }
}
