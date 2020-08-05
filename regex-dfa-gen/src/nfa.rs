//! Non-determinative Finite Automata
//! 
//! usage:
//! 
//! ```
//! use regex_dfa_gen::ast::AstNode;
//! use regex_dfa_gen::nfa::Nfa;
//!
//! let ast : AstNode = r"12".parse::<AstNode>().unwrap();
//! let nfa = Nfa::from_ast(&ast);
//! assert_eq!(nfa.len(), 2);
//! ```

use crate::ast::*;
use crate::set::*;
use std::borrow::Cow;

pub struct Nfa {
    pub(crate) states: Vec<NfaState>,
    pub(crate) node: NfaStateNode,
}
/// use `NfaBuilder::to_nfa` to get the nfa.
pub struct NfaBuilder {
    pub(crate) states: Vec<NfaState>,
}
/// can only use to a single builder.(will be improved)
pub struct NfaStateNode(pub(crate) Vec<usize>, Vec<usize>, bool);

#[derive(Clone, Debug)]
pub struct NfaState {
    pub(crate) ch : CharRange,
    pub(crate) table: Vec<usize>,
    pub(crate) is_greedy: bool,
    pub(crate) end_num: Option<usize>,
}


impl NfaState {
    #[inline]
    pub fn new(ch: CharRange, is_greedy: bool) -> Self {
        Self {
            ch,
            table: Vec::new(),
            is_greedy,
            end_num: None
        }
    }
}

impl NfaBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
        }
    }

    #[inline]
    fn push(&mut self,ch: CharRange, is_greedy: bool) -> usize {
        let ret = self.states.len();
        self.states.push(NfaState::new(ch, is_greedy));
        ret
    }

    #[inline]
    fn state_extend_all(&mut self, id_vec: &[usize], item: &[usize]) {
        for id in id_vec {
            self.states[*id].table.extend(item.iter().cloned());
        }
    }
    // returns can it be epsilon.
    fn build_from(&mut self, node: &AstNode, head: &mut Vec<usize>, tail: &mut Vec<usize>, is_greedy: bool) -> bool {
        use AstNode::*;
        match node {
            Char(ch) => {
                let id = self.push(ch.clone(), is_greedy);
                head.push(id);
                tail.push(id);
                false
            }
            Options(vec) => {
                let mut ret = false;
                for subnode in vec {
                    ret |= self.build_from(subnode, head, tail, is_greedy);
                }
                ret
            }
            Multiple(n) => {
                let hlen = head.len();
                let tlen = tail.len();
                // for `Multiple`, it can be epsilon.
                self.build_from(n, head, tail, true);
                self.state_extend_all(&tail[tlen..], &head[hlen..]);
                true
            }
            EmptyOr(n) => {
                self.build_from(n, head, tail, is_greedy);
                true
            }
            MultipleNonGreedy(n) => {
                let hlen = head.len();
                let tlen = tail.len();
                // for `Multiple`, it can be epsilon.
                self.build_from(n, head, tail, false);
                self.state_extend_all(&tail[tlen..], &head[hlen..]);
                true
            },
            Concat(vec) => {
                let mut tmp_head = Vec::new();
                let mut tmp_tail = Vec::new();
                let mut tmp = Vec::new();
                let mut first = true;
                for n in vec {
                    if first {
                        let hlen = head.len();
                        let can_be_eps = self.build_from(n, head, &mut tmp, false);
                        first &= can_be_eps;
                        self.state_extend_all(&tmp_tail[..], &head[hlen..]);
                        if !can_be_eps {
                            tmp_tail.clear();
                        }
                        tmp_tail.append(&mut tmp);
                        tmp.clear();
                    } else {
                        let can_be_eps = self.build_from(n, &mut tmp_head, &mut tmp, false);
                        self.state_extend_all(&tmp_tail[..], &tmp_head[..]);
                        
                        if !can_be_eps {
                            tmp_tail.clear();
                        }
                        tmp_tail.append(&mut tmp);
                        tmp_head.clear();
                        tmp.clear();
                    }
                }
                tail.append(&mut tmp_tail);
                first
            }
        }
    }
    /// build nfa from AST.
    pub fn from_ast(&mut self, ast: &AstNode) -> NfaStateNode {
        let mut head = Vec::new();
        let mut tail = Vec::new();
        let can_be_eps = self.build_from(ast, &mut head, &mut tail, false);
        NfaStateNode(head, tail, can_be_eps)
    }
    /// set the end at the end of Nfa nodes.
    pub fn set_end(&mut self, node: &NfaStateNode, end_num: usize) {
        for &ind in &node.1 {
            let state = &mut self.states[ind];
            state.end_num = Some(end_num);
        }
    }
    /// equal to regex `( node1 | node2 ... )`.
    pub fn options(&mut self, nodes: Vec<NfaStateNode>) -> NfaStateNode {
        let mut head = Vec::<usize>::new();
        let mut tail = Vec::new();
        let mut can_be_eps = false;
        for NfaStateNode(mut h, mut t,eps) in nodes.into_iter() {
            head.append(&mut h);
            tail.append(&mut t);
            can_be_eps |= eps;
        }
        NfaStateNode(head, tail, can_be_eps)
    }

    pub fn to_nfa(self, node: NfaStateNode) -> Nfa {
        Nfa{
            states: self.states,
            node,
        }
    }
    pub fn len(&self) -> usize {
        self.states.len()
    }
}


impl Nfa {
    pub fn from_ast(ast: &AstNode) -> Self {
        let mut builder = NfaBuilder::new();
        let node = builder.from_ast(ast);
        Self {
            states: builder.states,
            node,
        }
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.states.len()
    }
}


type Nd = usize;
type Ed = (usize,usize);
use std::io;

impl Nfa {
    
    /// get the dot file.
    /// 
    /// ```
    /// use regex_dfa_gen::ast::AstNode;
    /// use regex_dfa_gen::nfa::Nfa;
    /// use std::fs::File;
    ///
    /// let ast : AstNode = r"([A-Z]*|A[a-z]*?)H".parse::<AstNode>().unwrap();
    /// let nfa = Nfa::from_ast(&ast);
    /// 
    /// let mut f = File::create("nfa.dot").unwrap();
    /// nfa.render_to(&mut f).expect("msg");
    /// ```
    pub fn render_to<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        dot::render(self, w)
    }
}


impl<'a> dot::Labeller<'a, Nd, Ed> for Nfa {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example1").unwrap() }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n)).unwrap()
    }
    fn node_label<'b>(&'b self, n: &Nd) -> dot::LabelText<'b> {
        let state = &self.states[*n];
        let is_start = self.node.0.iter().find(|&x| x == n).is_some();
        let is_start = if is_start {"(s)"} else { "" };
        let is_end = self.node.1.iter().find(|&x| x == n).is_some();
        let is_end = if is_end {"(e)"} else { "" };

        dot::LabelText::LabelStr(format!("{}{}{}", show_char_range(state.ch.clone()), is_start, is_end).into())
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Nfa {
    fn nodes(&self) -> dot::Nodes<'a,Nd> {
        let nodes: Vec<usize> = (0..self.states.len()).collect();
        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a, Ed> {
        self.states.iter().enumerate()
            .flat_map(|(i, x)| x.table.iter().map(
                move |&j| (i, j)
            )).collect()
    }

    fn source(&self, e: &Ed) -> Nd { e.0 }

    fn target(&self, e: &Ed) -> Nd { e.1 }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::assert_eq;

    #[test]
    fn test0() {
        let ast : AstNode = r"12".parse::<AstNode>().unwrap();
        let nfa = Nfa::from_ast(&ast);
        assert_eq!(nfa.states.len(), 2);


        let ast : AstNode = r"1|2*3(5|4)*".parse::<AstNode>().unwrap();
        let nfa = Nfa::from_ast(&ast);
        assert_eq!(nfa.states.len(), 5);

        
        let ast : AstNode = r"([A-Za-z])(1?|2*3?(5|4)*)(e)".parse::<AstNode>().unwrap();
        let nfa = Nfa::from_ast(&ast);
        assert_eq!(nfa.states.len(), 8);
    }
}