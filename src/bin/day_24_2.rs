extern crate regex;

use regex::Regex;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

type GenError = Box<std::error::Error>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
enum GroupType {
    Immune,
    Infection,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Group {
    id: u32,
    size: i32,
    hit_points: i32,
    weaknesses: Vec<String>, // Sloppy, could enumerate all in example?
    immunities: Vec<String>,
    attack_power: i32,
    attack_type: String,
    group_type: GroupType,
    initiative: i32,
}

impl Group {
    fn effective_power(&self) -> i32 {
        self.attack_power * self.size
    }
    fn damage_dealt(&self, other: &Group) -> i32 {
        if other.immunities.contains(&self.attack_type) {
            return 0;
        }
        let base_damage = self.effective_power();
        if other.weaknesses.contains(&self.attack_type) {
            return base_damage * 2;
        }
        base_damage
    }
}

fn read_groups_from_file(path: &str) -> Result<Vec<Group>, GenError> {
    let f = File::open(path)?;
    let r = BufReader::new(f);

    // 72 units each with 5294 hit points (weak to slashing; immune to radiation, cold) with an attack that does 639 fire damage at initiative 1
    let re = Regex::new(r"(?P<size>\d+) units each with (?P<hit_points>\d+) hit points(?: \((((; )?weak to (?P<weaknesses>[^;)]+))|((; )?immune to (?P<immunities>[^;)]+))){1,2}\))? with an attack that does (?P<attack_power>\d+) (?P<attack_type>\w+) damage at initiative (?P<initiative>\d+)").unwrap();

    let mut groups: Vec<Group> = Vec::new();
    let mut current_group_type = GroupType::Immune;

    let mut immune_idx = 1;
    let mut infection_idx = 1;

    for line_result in r.lines() {
        let line = line_result?;

        if line.starts_with("Immune System:") {
            current_group_type = GroupType::Immune;
        } else if line.starts_with("Infection:") {
            current_group_type = GroupType::Infection;
        } else if let Some(cap) = re.captures(&line) {
            let mut group = Group {
                id: if current_group_type == GroupType::Immune {
                    immune_idx += 1;
                    immune_idx - 1
                } else {
                    infection_idx += 1;
                    infection_idx - 1
                },
                size: cap["size"].parse().unwrap(),
                hit_points: cap["hit_points"].parse().unwrap(),
                attack_power: cap["attack_power"].parse().unwrap(),
                weaknesses: if let Some(weaknesses) = cap.name("weaknesses") {
                    weaknesses
                        .as_str()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect()
                } else {
                    Vec::new()
                },
                immunities: if let Some(immunities) = cap.name("immunities") {
                    immunities
                        .as_str()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect()
                } else {
                    Vec::new()
                },
                attack_type: cap["attack_type"].to_string(),
                initiative: cap["initiative"].parse().unwrap(),
                group_type: current_group_type,
            };

            groups.push(group);
        }
    }

    Ok(groups)
}

fn play_game(groups: &mut Vec<Group>) -> (i32, Option<GroupType>) {
    let mut targetted = HashSet::new();
    let max_id = groups.len(); // sloppy.
    loop {
        // --------------------
        // Targeting phase
        // --------------------
        let mut targetting = vec![vec![std::u32::MAX; max_id]; 2];
        // targetting[0] == Immune
        // targetting[1] == Infection

        targetted.clear();

        // Sort by Effective power / initiative (both DESC)
        groups.sort_unstable_by(|g1, g2| {
            (g2.effective_power())
                .cmp(&g1.effective_power())
                .then_with(|| g2.initiative.cmp(&g1.initiative))
        });

        // println!("Targetting phase");
        // println!("{:?}", groups);

        // Choose the most damage dealt (ignoring overkill) / effectiver power / initiative
        // Can't choose group already targeted by someone else.
        for source_idx in 0..groups.len() {
            let source_group = &groups[source_idx];
            let mut max_damage_dealt = 0;
            let mut selected_target_idx = 0;
            for (target_idx, target_group) in groups.iter().enumerate() {
                if target_group.group_type == source_group.group_type
                    || targetted.contains(&target_idx)
                {
                    continue;
                }
                let damage_dealt = source_group.damage_dealt(target_group);
                if damage_dealt > max_damage_dealt {
                    max_damage_dealt = damage_dealt;
                    selected_target_idx = target_idx;
                }
                // For damage_dealt == max_damage_dealt we should check effective power / initiative as tie breakers...
                // But since we've already sorted on effective power / initiative in desc order, all other targets with same damage_dealt will have lower in some of those.
            }
            if max_damage_dealt > 0 {
                let source_group_type_idx = if source_group.group_type == GroupType::Immune {
                    0
                } else {
                    1
                };
                targetting[source_group_type_idx][source_group.id as usize] =
                    groups[selected_target_idx].id;
                targetted.insert(selected_target_idx);
            }
        }

        // println!("{:?}", targetted);
        // println!("{:?}", targetting);

        // --------------------
        // Attack phase
        // --------------------

        // Sort by initiative, max first
        groups.sort_unstable_by_key(|g| -g.initiative);

        // println!("Attack phase");
        // println!("{:?}", groups);

        // Give damage.
        let mut any_units_lost = false;
        for source_idx in 0..groups.len() {
            if groups[source_idx].size <= 0 {
                continue;
            }
            let source_group_type_idx: usize = if groups[source_idx].group_type == GroupType::Immune
            {
                0
            } else {
                1
            };

            let target_id = targetting[source_group_type_idx][groups[source_idx].id as usize];
            let enemy_group_type = if groups[source_idx].group_type == GroupType::Immune {
                GroupType::Infection
            } else {
                GroupType::Immune
            };
            if let Some(target_idx) = groups
                .iter()
                .position(|g| g.group_type == enemy_group_type && g.id == target_id)
            {
                let damage = groups[source_idx].damage_dealt(&groups[target_idx]);
                let units_lost = damage / groups[target_idx].hit_points;
                if units_lost > 0 {
                    any_units_lost = true;
                }

                if units_lost >= groups[target_idx].size {
                    // println!(
                    //     "{:?}:{} Does {} damage killing all {} units against {:?}:{}",
                    //     groups[source_idx].group_type,
                    //     groups[source_idx].id,
                    //     damage,
                    //     groups[target_idx].size,
                    //     groups[target_idx].group_type,
                    //     groups[target_idx].id
                    // );
                } else {
                    // println!(
                    //     "{:?}:{} Does {} damage killing {} units against {:?}:{}",
                    //     groups[source_idx].group_type,
                    //     groups[source_idx].id,
                    //     damage,
                    //     units_lost,
                    //     groups[target_idx].group_type,
                    //     groups[target_idx].id
                    // );
                }

                groups[target_idx].size -= units_lost;
            }
        }

        if !any_units_lost {
            return (0, None);
        }
        // Remove all dead groups.
        groups.retain(|ref g| g.size > 0);

        // Both groups left?
        if groups.iter().all(|i| i.group_type == GroupType::Immune) {
            return (groups.iter().map(|g| g.size).sum(), Some(GroupType::Immune));
        } else if groups.iter().all(|i| i.group_type == GroupType::Infection) {
            return (
                groups.iter().map(|g| g.size).sum(),
                Some(GroupType::Infection),
            );
        }
    }
}

fn find_minimum_boost(orig_groups: &[Group]) -> i32 {
    let mut boost = 0;
    loop {
        let mut boosted_groups: Vec<Group> = orig_groups.to_owned();
        for group in &mut boosted_groups {
            if group.group_type == GroupType::Immune {
                group.attack_power += boost;
            }
        }

        let (units, group_type) = play_game(&mut boosted_groups);

        if units == 0 {
            println!("Boost {}, Stalemate!", boost);
        } else if units > 0 {
            println!(
                "Boost {} : {:?} won with {} units left",
                boost,
                group_type.unwrap(),
                units
            );

            if group_type == Some(GroupType::Immune) {
                return units;
            }
        }

        boost += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_24_2() {
        let groups = read_groups_from_file("src/res/day_24.txt").unwrap();
        assert_eq!(1045, find_minimum_boost(&groups));
    }
}

fn main() {
    let groups = read_groups_from_file("src/res/day_24.txt").unwrap();
    println!("day_24_2 : minimum boost {:?}", find_minimum_boost(&groups));
}
