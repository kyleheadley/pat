// seems simple enough...
pub trait AsPattern<T> {
	fn pat(self) -> T;
}

// maybe this too?
// pub trait AsPatternRef<T> {
// 	fn pat_ref(&self) -> T;	
// }

// so we can eventually omit the `.pat()` in pattern matches
impl<T> AsPattern<T> for T {
	fn pat(self) -> T { self }
}

#[cfg(test)]
mod tests {
	mod list {
		use super::super::AsPattern;
		use std::rc::Rc;

		// simple abstractions
		pub struct List<T:Clone>(Link<T>);
		pub fn nil<T:Clone>() -> List<T> { List(None) }
		pub fn cons<T:Clone>(elm: T, list: &List<T>) -> List<T> {
			let List(ref link) = *list;
			List(Some(Rc::new(Node{elm:elm.clone(),next:link.clone()})))
		}

		// deconstruction patterns
		pub enum L<T:Clone> { Nil, Cons(T,List<T>)}
		pub struct Val<T>(pub T);
		pub enum Count<T> {None, Single(T), Double(T,T), Triple(T,T,T), TooMany }

		// efficient implementation
		type Link<T> = Option<Rc<Node<T>>>;
		struct Node<T:Clone> { elm: T, next: Link<T> }
		
		// deconstructon conversions (need case-by-case compiler optimizations on use)
		impl<T:Clone> AsPattern<L<T>> for List<T> {
			fn pat(self) -> L<T> {
				let List(link) = self;
				match link {
					None => return L::Nil,
					Some(n) => match *n {
						Node{ref elm, ref next} => L::Cons(elm.clone(),List(next.clone()))
					}
				}
			}
		}
		impl<T:Clone> AsPattern<Val<T>> for List<T> {
			fn pat(self) -> Val<T> {
				Val(self.head())
			}
		}
		impl<T:Clone> AsPattern<Count<T>> for List<T> {
			fn pat(self) -> Count<T> {
				let List(link) = self;
				match link {
					None => Count::None,
					Some(ref n) => match **n { Node{elm:ref elm1, ref next} => { match *next {
						None => Count::Single(elm1.clone()),
						Some(ref n) => match **n { Node{elm:ref elm2, ref next} => { match *next {
							None => Count::Double(elm1.clone(),elm2.clone()),
							Some(ref n) => match **n { Node{elm:ref elm3, ref next} => { match *next {
								None => Count::Triple(elm1.clone(),elm2.clone(), elm3.clone()),
								Some(..) => Count::TooMany,
							}}}
						}}}
					}}}
				}
			}
		}

		impl<T:Clone> List<T>{
			pub fn head(&self) -> T {
				let List(ref link) = *self;
				match *link {
					None => panic!("Attempt to access element of empty list"),
					Some(ref n) => match **n {
						Node{ref elm, ..} => elm.clone(),
					}
				}
			}
		}

		impl<T:Clone> Clone for List<T> {
			fn clone(&self) -> Self {
				let List(ref link) = *self;
				List(link.clone())
			}
		}
	}

	use super::*;
	// fns
	use self::list::{nil,cons};
	// patterns
	use self::list::{L,Val,Count};

    #[test]
    fn it_works() {

    	// really simple code

    	let a = nil();
    	let b = cons(1,&a);
    	let c = cons(2,&b);

    	// abstraction over implementation structs
    	match c.clone().pat() {
    		L::Nil => println!("oops!"),
    		L::Cons(e,l) => {
    			print!("Got {:?}, ", e);
    			match l.pat() {
    				L::Nil => println!("and that's the end"),
    				// will the compiler optimise away the unneeded conversions?
    				L::Cons(..) => println!("and more numbers"),
    			}
    		},
    	}

    	// with compiler support for the trait we could even rewrite the above as:
    	// match c.clone() {
    	// 	L::Nil => println!("oops!"),
    	// 	L::Cons(e,L::Nil) => println!("Got {:?}, and that's the end",e),
    	// 	L::Cons(e,L::Cons(..)) => println!("Got {:?}, and more numbers",e),
    	// }

    	// automatically converts to the matched items
    	match c.clone().pat() {
    		Count::None => println!("there was supposed to be something..."),
    		Count::Single(e) => println!("wasn't there more than just {:?}?", e),
    		Count::Double(e1,e2) => println!("we have our {:?} and {:?}", e1,e2),
    		Count::Triple(e1,e2,e3) => println!("{:?},{:?}, and {:?} seem like a lot", e1,e2,e3),
    		Count::TooMany => println!("we never expected that!"),
    	}

    	// not just style, enforces move (can't reverse these)
    	let _head = c.head();
    	let Val(_head) = c.pat();

    	println!("and we got our {:?} another way!", _head);

    }
}
