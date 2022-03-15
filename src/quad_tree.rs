use std::rc::Rc;

struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Rect {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Rect{
        Rect { x, y, width, height }
    }
}

const TL_INDEX: usize = 0;
const TR_INDEX: usize = 1;
const BL_INDEX: usize = 2;
const BR_INDEX: usize = 3;

fn index_rect_of_child(rect: &Rect, position: (f32, f32)) -> (usize, Rect) {
    let x0 = rect.x;
    let y0 = rect.y;
    let w = rect.width * 0.5;
    let h = rect.height * 0.5;
    let x1 = rect.x + w;
    let y1 = rect.y + h;

    if position.1 > y1 {
        if position.0 < x1 {
            (TL_INDEX, Rect::new(x0, y1, w, h))
        } else {
            (TR_INDEX, Rect::new(x1, y1, w, h))
        }
    } else {
        if position.0 < x1 {
            (BL_INDEX, Rect::new(x0, y0, w, h))
        } else {
            (BR_INDEX, Rect::new(x1, y0, w, h))
        }
    }
}

enum Node<D> {
    Leaf(Rc<D>),
    Nodes(Box<[Node<D>;4]>),
}

pub struct QuadTree<D: PartialEq> {
    root: Node<D>,
    rect: Rect,
    max_depth: i32,
}

impl<D: PartialEq> QuadTree<D> {
    pub fn get_data(&self, position: (f32, f32)) -> &D {
        QuadTree::get_node_data(&self.root, &self.rect, position)
    }

    fn get_node_data<'a, 'b>(node: &'a Node<D>, rect: &'b Rect, position: (f32, f32)) -> &'a D {
        match node {
            Node::Leaf(data) => data,
            Node::Nodes(children) => {
                let (index, rect) = index_rect_of_child(rect, position);
                QuadTree::get_node_data(&children[index], &rect, position)
            },
        }
    }

    pub fn set_data(&mut self, position: (f32, f32), data: &Rc<D>) {
        QuadTree::set_node_data(&mut self.root, &self.rect, position, data, 0, self.max_depth);
    }

    fn set_node_data(node: &mut Node<D>, rect: &Rect, position: (f32, f32), new_data: &Rc<D>, depth: i32, max_depth: i32) -> bool {
        match node {
            Node::Leaf(data) => {
                if data == new_data { return false }

                if depth < max_depth {
                    let mut children = Box::new([
                        Node::Leaf(Rc::clone(data)),
                        Node::Leaf(Rc::clone(data)),
                        Node::Leaf(Rc::clone(data)),
                        Node::Leaf(Rc::clone(data))
                    ]);

                    let (index, rect) = index_rect_of_child(rect, position);
                    QuadTree::set_node_data(&mut children[index], &rect, position, new_data, depth+1, max_depth);

                    *node = Node::Nodes(children);

                    false
                } else {
                    *data = Rc::clone(new_data);
                    true
                }
            }
            Node::Nodes(children) => {
                let (index, rect) = index_rect_of_child(rect, position);
                if QuadTree::set_node_data(&mut children[index], &rect, position, new_data, depth+1, max_depth) {
                    let mut can_merge = true;
                    for i in 0..4 {
                        if let Node::Leaf(data) = &children[i] {
                            if data != new_data {
                                can_merge = false;
                            }
                        } else {
                            can_merge = false;
                        }
                    }

                    if can_merge {
                        *node = Node::Leaf(Rc::clone(new_data));
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }
}