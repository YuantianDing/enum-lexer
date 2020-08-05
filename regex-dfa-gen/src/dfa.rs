
use sorted_vec::SortedVec;
use crate::set::*;
use crate::nfa::*;
use std::borrow::Cow;
use std::collections::HashMap;
pub struct DfaBuilder<'a>{
    // SortedVec<usize> store nfa_states
    pub(crate) states: Vec<(DfaState, SortedVec<usize>)>,
    // pub(crate) hashmap: HashMap<DfaState, usize>,
    pub(crate) nfa: &'a Nfa
}
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct DfaState {
    // CharRange, usize(dfa_state), is_greedy
    pub table: Vec<(CharRange, usize, bool)>,
    pub end_num: Option<usize>
}


impl<'a> DfaBuilder<'a> {
    fn new(nfa: &'a Nfa) -> Self {
        Self {
            states: Vec::with_capacity(0),
            // hashmap: HashMap::new(),
            nfa,
        }
    }
    fn push(&mut self, state: DfaState, nfa_states: SortedVec<usize>) -> usize {
        let ret = self.states.len();
        self.states.push((state, nfa_states));
        ret
    }
    fn build(&mut self, nfa_states: SortedVec<usize>) -> usize {
        let ret = self.states.iter().position(|state| state.1 == nfa_states);
        // dbg!(&nfa_states, ret);
        if let Some(index) = ret {
            index
        } else {
            let iter: _ = nfa_states.iter().flat_map(|nfa_state| &self.nfa.states[*nfa_state].table).copied();
            let maps: _ = self.iter_to_map(iter);
            self.build_from_vec(nfa_states, maps)
        }
    }

    fn iter_to_map(&self, targets: impl Iterator<Item=usize>) -> RangeMap::<char, usize> {
        let mut maps = RangeMap::<char, usize>::new();
        for i in targets {
            let target = &self.nfa.states[i];
            maps.insert(target.ch.clone(), i);
        }
        maps
    }

    fn state_init(&mut self, nfa_states: SortedVec<usize>) -> usize {
        let end_num = nfa_states.iter()
        .filter_map(|&x| self.nfa.states[x].end_num)
        .max_by(|x,y| x.cmp(y));
        self.push(
            DfaState{
                table: Vec::new(),
                end_num,
            },
            nfa_states,
        )
    }

    #[inline]
    fn build_from_vec(&mut self, nfa_states: SortedVec<usize>, vec: RangeMap::<char, usize>) -> usize {
        let vec = vec.0;
        let ret = self.state_init(nfa_states);

        let mut table = Vec::new();
        for (k, v) in vec {
            let mut v = SortedVec::from_unsorted(v);
            v.dedup();
            let is_greedy = v.iter().map(|&i| self.nfa.states[i].is_greedy)
                .fold(false, |a,b| a | b);
            table.push((
                k, self.build(v), is_greedy
            ))
        }
        self.states[ret].0.table = table;

        ret
    }
    /// get the builder from nfa.
    pub fn from_nfa(nfa: &'a Nfa) -> Self {
        let mut ret = Self::new(&nfa);
        let maps = ret.iter_to_map(nfa.node.0.iter().copied());
        ret.build_from_vec(SortedVec::new(), maps);
        ret
    }
    
    /// get the dfa.
    pub fn to_dfa(self) -> Dfa {
        Dfa {
            states:
            self.states.into_iter().map(|(state, _)|
                state
            ).collect()
        }
    }
}


pub struct Dfa {
    pub states: Vec<DfaState>,
}


type Nd = (usize, Option<usize>);
type Ed = (usize, usize, CharRange, bool);
use std::io;

impl Dfa {
    /// get the dot file.
    /// 
    /// ```
    /// use regex_dfa_gen::ast::AstNode;
    /// use regex_dfa_gen::nfa::Nfa;
    /// use regex_dfa_gen::dfa::{ DfaBuilder, Dfa };
    /// use std::fs::File;
    /// 
    /// let ast : AstNode = r"([A-Z]*|A[a-z]*?)H".parse::<AstNode>().unwrap();
    /// let nfa = Nfa::from_ast(&ast);
    /// let dfa = DfaBuilder::from_nfa(&nfa).to_dfa();
    /// 
    /// let mut f = File::create("dfa.dot").unwrap();
    /// dfa.render_to(&mut f).expect("msg");
    /// ```
    pub fn render_to<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        dot::render(self, w)
    }

    pub fn from_nfa(nfa: &Nfa) -> Dfa {
        DfaBuilder::from_nfa(&nfa).to_dfa()
    }
    pub fn replace(self, pair: HashMap<usize, usize>) -> Dfa {
        let mut ret = Vec::new();
        let mut maps= Vec::new();

        let mut ind = 0;
        for i in 0..self.states.len() {
            if let Some(&j) = pair.get(&i) {
                maps.push(maps[j])
            } else {
                maps.push(ind);
                ind += 1;
            }
        }


        for (i,mut s) in self.states.into_iter().enumerate() {
            if !pair.contains_key(&i) {
                for (_, arc, _) in s.table.iter_mut() {
                    *arc = maps[*arc];
                }
                ret.push(s);
            }
        }
        Dfa{states: ret}
    }
    
    pub fn opt(self) -> Dfa {
        let mut maps = HashMap::new();
        let mut pair = HashMap::new();
        for (i, state) in self.states.iter().enumerate() {
            if let Some(&j) = maps.get(&state) {
                pair.insert(i, j);
            }
            maps.insert(state, i);
        }
        let ret = self.replace(pair);
        ret
    }
}


impl<'a> dot::Labeller<'a, Nd, Ed> for Dfa {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example1").unwrap() }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n.0)).unwrap()
    }

    fn node_label<'b>(&'b self, n: &Nd) -> dot::LabelText<'b> {
        let (i, num) = n;
        if let Some(n) = num {
            dot::LabelText::LabelStr(format!("{}({})", i, n).into())
        }else {
            dot::LabelText::LabelStr(format!("{}", i).into())
        }
    }
    fn edge_label<'b>(&'b self, (_, _, ch, _): &Ed) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(format!("{}", show_char_range(ch.clone())).into())
    }
    fn edge_color<'b>(&'b self, (_, _, _, is_greedy): &Ed) -> Option<dot::LabelText<'b>>{
        if *is_greedy {
            Some(dot::LabelText::LabelStr("red4".into()))
        } else {
            None
        }
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Dfa {
    fn nodes(&self) -> dot::Nodes<'a,Nd> {
        let nodes: Vec<_> = self.states.iter().enumerate()
            .map(|(i,state)| (i, state.end_num.clone()))
            .collect();
        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a, Ed> {
        self.states.iter().enumerate()
        .flat_map(|(i, x)| x.table.iter().map(
            move |(range, j, is_greedy)| (i, *j, range.clone(), *is_greedy)
        )).collect()
    }

    fn source(&self, e: &Ed) -> Nd { (e.0, self.states[e.0].end_num) }

    fn target(&self, e: &Ed) -> Nd { (e.1, self.states[e.1].end_num) }
}



#[cfg(test)]
mod test {
    use super::*;
    use std::assert_eq;
    use std::fs::File;
    use crate::ast::*;
    #[test]
    fn test0() {
        let ast: AstNode = r"12".parse::<AstNode>().unwrap();
        let nfa = Nfa::from_ast(&ast);
        let dfa = DfaBuilder::from_nfa(&nfa).to_dfa();
        assert_eq!(dfa.states.len(), 3);

        let ast: AstNode = r"([A-Z]*|A[a-z]*?)H".parse::<AstNode>().unwrap();
        let nfa = Nfa::from_ast(&ast);
        let dfa = DfaBuilder::from_nfa(&nfa).to_dfa();

        let mut f = File::create("dfa.dot").unwrap();
        dfa.render_to(&mut f).expect("msg");

        assert_eq!(nfa.states.len(), 4);
        assert_eq!(dfa.states.len(), 6);
    }
}