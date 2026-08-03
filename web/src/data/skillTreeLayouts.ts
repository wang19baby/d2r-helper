export interface SkillNodeDef {
  skillId: number
  icon: string
  x: number
  y: number
  edges: ('left' | 'right' | 'top')[]
  passive?: boolean
}

export interface SkillTreeDef {
  title: string
  tabBg?: string
  nodes: SkillNodeDef[]
}

function reqLevel(y: number): number {
  if (y < 15) return 1
  if (y < 33) return 6
  if (y < 50) return 12
  if (y < 65) return 18
  if (y < 80) return 24
  return 30
}

export function reqLevelForNode(node: SkillNodeDef): number {
  return reqLevel(node.y)
}

const S = 'https://d2emu.com/d2s/static/img/skill-tabs'

function n(skillId: number, icon: string, col: number, row: number, edges: ('left' | 'right' | 'top')[], passive?: boolean): SkillNodeDef {
  const xs = [0, 19.8, 50.0, 80.2]
  const ys = [0, 8.7, 24.7, 40.7, 57.4, 73.5, 89.6]
  return { skillId, icon, x: xs[col], y: ys[row], edges, passive }
}

export const SKILL_TREE_LAYOUTS: Record<string, SkillTreeDef[]> = {
  Amazon: [
    { title: 'Bow and Crossbow Skills (Amazon)', nodes: [
      n(0, 'MA', 2, 1, ['top']),
      n(1, 'FA', 3, 1, ['top', 'right']),
      n(2, 'CA', 1, 2, ['left']),
      n(3, 'MS', 2, 2, ['top']),
      n(4, 'EA', 3, 3, ['right']),
      n(5, 'IA', 1, 4, ['left']),
      n(6, 'GA', 2, 4, ['top']),
      n(7, 'ST', 2, 5, ['top']),
      n(8, 'IM', 3, 5, ['right']),
      n(9, 'FZ', 1, 6, ['left']),
    ]},
    { title: 'Passive and Magic Skills (Amazon)', nodes: [
      n(10, 'IS', 1, 1, ['top', 'left']),
      n(11, 'CR', 3, 1, ['top', 'right']),
      n(12, 'DG', 2, 2, ['top']),
      n(13, 'SM', 1, 3, ['left']),
      n(14, 'AV', 2, 3, ['top']),
      n(15, 'PN', 3, 4, ['right']),
      n(16, 'DE', 1, 5, ['left']),
      n(17, 'EV', 2, 5, ['top']),
      n(18, 'VK', 1, 6, ['left']),
      n(19, 'PC', 3, 6, ['right']),
    ]},
    { title: 'Javelin and Spear Skills (Amazon)', nodes: [
      n(20, 'JB', 1, 1, ['top', 'left']),
      n(21, 'PS', 2, 2, ['top']),
      n(22, 'PJ', 3, 2, ['right']),
      n(23, 'IP', 1, 3, ['left']),
      n(24, 'LB', 3, 3, ['right']),
      n(25, 'CS', 2, 4, ['top']),
      n(26, 'PL', 3, 4, ['right']),
      n(27, 'FE', 1, 5, ['left']),
      n(28, 'LS', 2, 6, ['top']),
      n(29, 'LF', 3, 6, ['right']),
    ]},
  ],

  Sorceress: [
    { title: 'Fire Skills (Sorceress)', tabBg: `${S}/sorceress/Fire.png`, nodes: [
      n(0, 'FB', 2, 1, ['top']),
      n(1, 'WA', 3, 1, ['top', 'right']),
      n(2, 'IN', 1, 2, ['left']),
      n(3, 'BL', 1, 3, ['left']),
      n(4, 'FL', 2, 3, ['top']),
      n(5, 'FW', 1, 4, ['left']),
      n(6, 'EN', 3, 4, ['right']),
      n(7, 'ME', 2, 5, ['top']),
      n(8, 'FM', 2, 6, ['top']),
      n(9, 'HY', 3, 6, ['right']),
    ]},
    { title: 'Lightning Skills (Sorceress)', tabBg: `${S}/sorceress/Lightning.png`, nodes: [
      n(10, 'CB', 2, 1, ['top']),
      n(11, 'SF', 1, 2, ['left']),
      n(12, 'TK', 3, 2, ['right']),
      n(13, 'NV', 1, 3, ['left']),
      n(14, 'LG', 2, 3, ['top']),
      n(15, 'CL', 2, 4, ['top']),
      n(16, 'TP', 3, 4, ['right']),
      n(17, 'TS', 1, 5, ['left']),
      n(18, 'ES', 3, 5, ['right']),
      n(19, 'LM', 2, 6, ['top']),
    ]},
    { title: 'Cold Skills (Sorceress)', tabBg: `${S}/sorceress/Cold.png`, nodes: [
      n(20, 'IB', 2, 1, ['top']),
      n(21, 'FA', 3, 1, ['top', 'right']),
      n(22, 'FN', 1, 2, ['left']),
      n(23, 'IBl', 2, 2, ['top']),
      n(24, 'SA', 3, 3, ['right']),
      n(25, 'GS', 2, 4, ['top']),
      n(26, 'BZ', 1, 5, ['left']),
      n(27, 'CA', 3, 5, ['right']),
      n(28, 'CM', 2, 6, ['top']),
      n(29, 'FO', 1, 6, ['left']),
    ]},
  ],

  Necromancer: [
    { title: 'Curses (Necromancer)', tabBg: `${S}/necromancer/Curses.png`, nodes: [
      n(0, 'AD', 2, 1, ['top']),
      n(1, 'DV', 1, 2, ['left']),
      n(2, 'WE', 3, 2, ['right']),
      n(3, 'IM', 2, 3, ['top']),
      n(4, 'TE', 3, 3, ['right']),
      n(5, 'CO', 1, 4, ['left']),
      n(6, 'LT', 2, 4, ['top']),
      n(7, 'AT', 1, 5, ['left']),
      n(8, 'DE', 3, 5, ['right']),
      n(9, 'LR', 2, 6, ['top']),
    ]},
    { title: 'Poison and Bone Skills (Necromancer)', nodes: [
      n(10, 'TH', 2, 1, ['top']),
      n(11, 'BA', 3, 1, ['top', 'right']),
      n(12, 'PD', 1, 2, ['left']),
      n(13, 'CE', 2, 2, ['top']),
      n(14, 'BW', 3, 3, ['right']),
      n(15, 'PE', 1, 4, ['left']),
      n(16, 'BS', 2, 4, ['top']),
      n(17, 'BP', 3, 5, ['right']),
      n(18, 'PN', 1, 6, ['left']),
      n(19, 'BSp', 2, 6, ['top']),
    ]},
    { title: 'Summoning Skills (Necromancer)', nodes: [
      n(20, 'SM', 1, 1, ['top', 'left']),
      n(21, 'RS', 3, 1, ['top', 'right']),
      n(22, 'CG', 2, 2, ['top']),
      n(23, 'GM', 1, 3, ['left']),
      n(24, 'RSM', 3, 3, ['right']),
      n(25, 'BG', 2, 4, ['top']),
      n(26, 'SR', 1, 5, ['left']),
      n(27, 'IG', 2, 5, ['top']),
      n(28, 'FG', 2, 6, ['top']),
      n(29, 'RV', 3, 6, ['right']),
    ]},
  ],

  Paladin: [
    { title: 'Combat Skills (Paladin)', tabBg: `${S}/paladin/Combat.png`, nodes: [
      n(0, 'SA', 1, 1, ['top', 'left']),
      n(1, 'SM', 3, 1, ['top', 'right']),
      n(2, 'HB', 2, 2, ['top']),
      n(3, 'ZL', 1, 3, ['left']),
      n(4, 'CH', 3, 3, ['right']),
      n(5, 'VG', 1, 4, ['left']),
      n(6, 'BH', 2, 4, ['top']),
      n(7, 'CV', 1, 5, ['left']),
      n(8, 'HS', 3, 5, ['right']),
      n(9, 'FH', 2, 6, ['top']),
    ]},
    { title: 'Offensive Auras (Paladin)', nodes: [
      n(10, 'MI', 1, 1, ['top', 'left']),
      n(11, 'HF', 2, 2, ['top']),
      n(12, 'TH', 3, 2, ['right']),
      n(13, 'BA', 1, 3, ['left']),
      n(14, 'CN', 1, 4, ['left']),
      n(15, 'HZ', 2, 4, ['top']),
      n(16, 'HSa', 2, 5, ['top']),
      n(17, 'SC', 3, 5, ['right']),
      n(18, 'FN', 1, 6, ['left']),
      n(19, 'CV', 3, 6, ['right']),
    ]},
    { title: 'Defensive Auras (Paladin)', nodes: [
      n(20, 'PR', 1, 1, ['top', 'left']),
      n(21, 'RF', 3, 1, ['top', 'right']),
      n(22, 'DF', 2, 2, ['top']),
      n(23, 'RC', 3, 2, ['right']),
      n(24, 'CL', 1, 3, ['left']),
      n(25, 'RL', 3, 3, ['right']),
      n(26, 'VG', 2, 4, ['top']),
      n(27, 'MD', 1, 5, ['left']),
      n(28, 'RD', 2, 6, ['top']),
      n(29, 'SV', 3, 6, ['right']),
    ]},
  ],

  Barbarian: [
    { title: 'Combat Skills (Barbarian)', tabBg: `${S}/barbarian/Combat.png`, nodes: [
      n(0, 'BA', 2, 1, ['top']),
      n(1, 'LE', 1, 2, ['left']),
      n(2, 'DS', 3, 2, ['right']),
      n(3, 'ST', 2, 3, ['top']),
      n(4, 'DT', 3, 3, ['right']),
      n(5, 'LA', 1, 4, ['left']),
      n(6, 'CN', 2, 4, ['top']),
      n(7, 'FR', 3, 5, ['right']),
      n(8, 'BE', 2, 6, ['top']),
      n(9, 'WW', 1, 6, ['left']),
    ]},
    { title: 'Masteries (Barbarian)', nodes: [
      n(10, 'BM', 1, 1, ['top', 'left']),
      n(11, 'AM', 2, 1, ['top']),
      n(12, 'MM', 3, 1, ['top', 'right']),
      n(13, 'PM', 1, 2, ['left']),
      n(14, 'TM', 2, 2, ['top']),
      n(15, 'SpM', 3, 2, ['right']),
      n(16, 'IS', 1, 3, ['left']),
      n(17, 'IZ', 3, 4, ['right']),
      n(18, 'IiS', 1, 5, ['left']),
      n(19, 'NR', 3, 6, ['right']),
    ]},
    { title: 'Warcries (Barbarian)', nodes: [
      n(20, 'HO', 1, 1, ['top', 'left']),
      n(21, 'FP', 3, 1, ['top', 'right']),
      n(22, 'TA', 1, 2, ['left']),
      n(23, 'SH', 2, 2, ['top']),
      n(24, 'FI', 3, 3, ['right']),
      n(25, 'BC', 1, 4, ['left']),
      n(26, 'BO', 2, 5, ['top']),
      n(27, 'GW', 3, 5, ['right']),
      n(28, 'WC', 1, 6, ['left']),
      n(29, 'BCo', 2, 6, ['top']),
    ]},
  ],

  Druid: [
    { title: 'Summoning Skills (Druid)', tabBg: `${S}/druid/Summoning.png`, nodes: [
      n(0, 'RN', 2, 1, ['top']),
      n(1, 'PC', 3, 1, ['top', 'right']),
      n(2, 'OS', 1, 2, ['left']),
      n(3, 'SSW', 2, 2, ['top']),
      n(4, 'CV', 3, 3, ['right']),
      n(5, 'HW', 1, 4, ['left']),
      n(6, 'SDW', 2, 4, ['top']),
      n(7, 'SC', 3, 5, ['right']),
      n(8, 'SB', 1, 6, ['left']),
      n(9, 'SG', 2, 6, ['top']),
    ]},
    { title: 'Shape Shifting Skills (Druid)', nodes: [
      n(10, 'WW', 1, 1, ['top', 'left']),
      n(11, 'LY', 2, 1, ['top']),
      n(12, 'WB', 3, 2, ['right']),
      n(13, 'FR', 1, 3, ['left']),
      n(14, 'MA', 3, 3, ['right']),
      n(15, 'RA', 1, 4, ['left']),
      n(16, 'FC', 2, 4, ['top']),
      n(17, 'HU', 2, 5, ['top']),
      n(18, 'SH', 3, 5, ['right']),
      n(19, 'FU', 1, 6, ['left']),
    ]},
    { title: 'Elemental Skills (Druid)', nodes: [
      n(20, 'FS', 1, 1, ['top', 'left']),
      n(21, 'MB', 1, 2, ['left']),
      n(22, 'AB', 3, 2, ['right']),
      n(23, 'FI', 1, 3, ['left']),
      n(24, 'CA', 3, 3, ['right']),
      n(25, 'TW', 2, 4, ['top']),
      n(26, 'VC', 1, 5, ['left']),
      n(27, 'TN', 2, 5, ['top']),
      n(28, 'AG', 1, 6, ['left']),
      n(29, 'HU', 2, 6, ['top']),
    ]},
  ],

  Assassin: [
    { title: 'Traps (Assassin)', tabBg: `${S}/assassin/Traps.png`, nodes: [
      n(0, 'FT', 2, 1, ['top']),
      n(1, 'SW', 1, 2, ['left']),
      n(2, 'BS', 3, 2, ['right']),
      n(3, 'CBS', 1, 3, ['left']),
      n(4, 'WoF', 2, 3, ['top']),
      n(5, 'BF', 3, 4, ['right']),
      n(6, 'LS', 1, 5, ['left']),
      n(7, 'WoI', 2, 5, ['top']),
      n(8, 'DS', 1, 6, ['left']),
      n(9, 'BSh', 3, 6, ['right']),
    ]},
    { title: 'Shadow Disciplines (Assassin)', nodes: [
      n(10, 'CM', 2, 1, ['top']),
      n(11, 'PH', 3, 1, ['top', 'right']),
      n(12, 'BoS', 1, 2, ['left']),
      n(13, 'WB', 2, 3, ['top']),
      n(14, 'CoS', 3, 3, ['right']),
      n(15, 'FA', 1, 4, ['left']),
      n(16, 'SWa', 2, 4, ['top']),
      n(17, 'MB', 3, 5, ['right']),
      n(18, 'VM', 1, 6, ['left']),
      n(19, 'SMa', 2, 6, ['top']),
    ]},
    { title: 'Martial Arts (Assassin)', nodes: [
      n(20, 'TS', 2, 1, ['top']),
      n(21, 'DT', 3, 1, ['top', 'right']),
      n(22, 'FoF', 1, 2, ['left']),
      n(23, 'DC', 3, 2, ['right']),
      n(24, 'CS', 2, 3, ['top']),
      n(25, 'CoT', 1, 4, ['left']),
      n(26, 'DTa', 3, 4, ['right']),
      n(27, 'BoI', 1, 5, ['left']),
      n(28, 'DF', 3, 5, ['right']),
      n(29, 'PS', 2, 6, ['top']),
    ]},
  ],

  Warlock: [
    { title: 'Sigils (Warlock)', tabBg: '/assets/img/skill-tabs/warlock/Chaos.png', nodes: [
      n(22, 'BO', 3, 1, ['top'], true),
      n(21, 'SG', 1, 2, ['top']),
      n(20, 'DM', 2, 2, ['top'], true),
      n(23, 'DMa', 2, 3, ['top']),
      n(24, 'ST', 3, 5, ['top']),
      n(25, 'BB', 1, 4, ['top']),
      n(26, 'SD', 3, 3, ['top']),
      n(27, 'EN', 2, 5, ['top']),
      n(28, 'CO', 1, 6, ['top']),
      n(29, 'BD', 3, 6, ['top']),
    ]},
    { title: 'Eldritch (Warlock)', tabBg: '/assets/img/skill-tabs/warlock/Eldritch.png', nodes: [
      n(10, 'LM', 1, 1, ['top'], true),
      n(12, 'CL', 3, 1, ['top']),
      n(17, 'EB', 2, 2, ['top']),
      n(13, 'ES', 1, 3, ['top']),
      n(14, 'HP', 3, 3, ['top']),
      n(15, 'BW', 1, 4, ['top']),
      n(16, 'PW', 2, 4, ['top']),
      n(11, 'HB', 2, 5, ['top']),
      n(18, 'HS', 3, 5, ['top']),
      n(19, 'MBl', 2, 6, ['top']),
    ]},
    { title: 'Summoning (Warlock)', tabBg: '/assets/img/skill-tabs/warlock/Demon.png', nodes: [
      n(1, 'RO', 1, 1, ['top']),
      n(0, 'MB', 3, 1, ['top']),
      n(5, 'FW', 1, 2, ['top']),
      n(2, 'SL', 2, 2, ['top']),
      n(3, 'SR', 3, 3, ['top']),
      n(7, 'EE', 2, 4, ['top'], true),
      n(4, 'MC', 3, 4, ['top']),
      n(6, 'SD', 2, 5, ['top']),
      n(8, 'AP', 1, 6, ['top']),
      n(9, 'AB', 3, 6, ['top']),
    ]},
  ],
}
