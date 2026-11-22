                                                        /**/#![doc=               /**/concat!               /********/(               " Snowflakes with "       /********/,
                                                      "six-fold symmetry, "     /************/,           /*************/           /*************/           /*************/
                                                     "drawn as text.",         /***/)]#![doc=""]         /**/#![doc=concat         /*************/!(         " One arm is grown on "
                                                    /****************/,       "a triangular lattice "   ,"and then copied "       ,"through the 12",)       /**/]#![doc=concat!
                                                    /*****************/(     /**/" rotations and "     ,"reflections of a ",     "hexagon. The arm's "     /*****************/,
                                                    "shape comes from a "   /***/,"short",)]#![doc=   /*************/concat!(   " list of \"weather\" "   /******************/,
                                                     /*******************/   "values, one per step " ,"out from the centre. ", /**/"A terminal",)]#![doc /**********/=concat!(
                                                      " cell is about twice ","as tall as it is ","wide, so the lattice ","fits the",)]#![doc=concat!(" character grid with "
                                                       /****************/,"very little ","distortion.",)]#![doc=""]#![doc=concat!(" Every list of ","weather values gives ",
                                                        /********/"a different flake, so ","each flake can",)]#![doc=concat!(" have a `u64` ID. Up ","to size 24 the IDs ",
                                                         /**********/"count the flakes ","exactly, from 0 to",)]#![doc=concat!(" one less than the ","number of flakes. ",
                                                          /*********/"Past that there are ","more flakes than",)]#![doc=concat!(" `u64` values, so ","every `u64` is a ",
                                                           /************/"valid ID but most ","flakes don't have one.",)]#![doc=""]#![doc=" ```"]#![doc=concat!(" use ",
                                           /*****/"asciiflake::{Snowflake",", Style};",)]#![doc=""]#![doc=concat!(" ","assert_eq!(Snowflake::","count(6), Some(144));",)]#![doc=concat!(
                                         /****************************************/" let flake = ","Snowflake::from_id(6, ","42).unwrap();",)]#![doc=concat!(" ","println!(\"{flake}\");",
                                        "                      ","   // ASCII",)]#![doc /**********/=concat!(" ", "println!(\"{flake:#}\"", /**/");                    ","   // Unicode",)]
                                       /*********/#![doc=concat!(" println!(\"{}\", ",   /*********************/   /*********************/   /********************/"flake.render(Style::UN",
                                       /**/"ICODE)); // same, ","explicitly",)]#![doc     /**********/=concat!(     " println!(\"{}\", ",     /*******************/"Snowflake::from_id(30,",
                                       " u64::MAX).unwrap());",)]#![doc=" ```"]use std   /***/::collections::{       /**/BTreeMap,BTreeSet   /****/};use std::fmt;use std::iter::successors;
                                        /******************/pub mod code;#[doc=concat!( /*******************/         /*******************/ /*********************/" A snowflake, stored ",
                                         "as the weather it ","grew through: one ","value per step out",)]#[           /*********/doc=concat!(" from the centre. ","`weather[t]` is how ",
                                          /*****/"far side branches ","that start at time",)]#[doc=concat!(             " `t` can grow, and ","zero means none start ","then. Different "
                                           ,"weather always",)]#     /**********************/[doc=concat!(               /*************/" grows a different ",     /*******************/
                                            /*******************/     /********/"flake, so this is all ",                 "a flake needs to hold.",)]#[derive     /******/(Clone,Debug,
                                             /**/PartialEq,Eq)]pub     struct Snowflake{weather:Vec<i32>                   /**/,}impl Snowflake{#[doc=concat     /*****************/!(
                                              " The biggest size ",     /****/"where every flake has ",                     "its own `u64` ID.",)]pub const     MAX_ID_SIZE:u32=24;#[
                              /************/doc=concat!(" How many different ","flakes there are of ",      /*********/      /*******************/"this size, or `None` ","past",)]#[doc=concat!(" ",
                            /************************/"[`MAX_ID_SIZE`](Self::","MAX_ID_SIZE) where ",     /*************/     /******/"the count doesn't fit ","in a `u64`.",)]pub fn count(size:u32)->
                           Option<u64>{(0..size).try_fold(1u64,|n,t|n.checked_mul(Self::choices(size     /***/,t)))}#[doc=     /******************/concat!(" The flake with this ","ID. Different IDs ",
                          /*************/"always give different ","flakes. If",)]#[doc=concat!(" ",     /*****************/     "[`count(size)`](Self::","count) is `Some(n)`, ","IDs run from 0 to `n ",
                          /*******************/"- 1` and",)]#[doc=concat!(" this returns `None` ",     /*******************/     /******/"for anything bigger. ","Otherwise every `u64` ","is a",)]#[doc=
                          /*****************/" valid ID."]#[doc=""]#[doc=concat!(" Listing every ",      "weather in order, "   /***************/,"with the value ","nearest the centre",)]#[doc=concat!(
                           /**********/" changing fastest, ","the ID is the ","weather's place in ", "that list. That's a",)]# /*************/[doc=concat!(" mixed-radix number ","with one digit per ",
                            "time, where digit `t` ","has base",)]#[doc=" `choices(size, t)`."]#[doc=""]#[doc=concat!(" When there are more ","flakes than `u64` ","values, using the ID ","directly",)
                             /********/]#[doc=concat!(" would only ever ","reach the start of ","the list, which ","leaves the outermost",)]#[doc=concat!(" digits stuck at ","zero. Instead the ID ",
                              /**/"is read as the "                               ,"fraction `id / 2^64`",)]#[doc=concat!(" of the way through ",                               /*******************/
                               /*******************/                               "the list, so IDs are ","spread evenly over ","all of it.",)]                               /*****/#[doc=concat!(
                                /*******************/                               /*********/" Each step multiplies ","the fraction by the ",                               /*******************/
                                 /*******************/                               /****/"next base and takes ","the whole",)]#[doc=concat!(                               /*******************/
                 /*************/" part as the digit, ",              /***********/"starting from the ","outermost. Because ","there are more",)]#[doc=concat!(              /*************/" places in the list ",
               "than IDs, two IDs ","never land on the "           /***/,"same one.",)]pub fn from_id(size:u32,id:u64)->Option<Self>{let mut weather=vec![0;size           /*****/as usize];match Self::count(size){
              Some(count)=>{if id>=count{return None;}let         /*****/mut rest=id;for(t,w)in(0..size).zip(&mut weather){*w=(rest%Self::choices(size,t))as i32;         /****/rest/=Self::choices(size,t);}}None=>{
             let mut fraction=id;for(t,w)in(0..size).zip(&       /****/mut weather).rev(){let scaled=u128::from(   fraction)*u128::from(Self::choices(size,t));*w=       (scaled>>64)as i32;fraction=scaled as u64;}}}
             /*****/Some(Snowflake{weather})}#[doc=concat!(      " Draw the flake with ","the given glyphs.",)]     /***/pub fn render(&self,style:Style)->String{      let mut cells=BTreeMap::<(i32,i32),u8>::new();
             for Edge(a,b)in self.edges(){let((x0,y0),(x1,y1     ))=(a.screen(),b.screen());let(sx,sy)=((x1-x0).   signum(),(y1-y0).signum());let axis=1<<(sx*sy+1     );for k in 0..=(x1-x0).abs().max((y1-y0).abs())
              {*cells.entry((y0+k*sy,x0+k*sx)).or_default()|=     axis;}}let reach= |f:fn(&(i32,i32))->i32|cells. /**/keys().map(f).max().unwrap_or(0);let(h,w)=(     reach(|c|c.0.abs()),reach(|c|c.1.abs()));(-h..=
               h).map(|y|{let row:String=(-w..=w).map(|x|style     /*********/.glyph(cells.get(&(y,x)))).collect();row.trim_end().to_owned()}).collect::<Vec<_>>     /*****************/().join("\n")}#[doc=concat!(
                /*******/" How many weather ","values make a ",     /************/"difference at time ","`t` in a flake of",)]#[doc=concat!(" this size. The ",     /**********************/"longest side branch ",
                 "that can start at ",     "time `t` is the one"     /**/,)]#[doc=concat!(     /************/" on the spine, with ",     /*******************/     /*******************/     /*******************/
                  /*******************/     /*******************/     /*******************/     /********/"room for `min(t, size ",        "- t)` steps, so "     ,"anything longer",)]     /*****/#[doc=concat!(
                   " acts the same as ",     /*******************/     /*******************/     /******/"that. Adding 1 counts ",     /*******************/     /*******************/     /*******************/
                    /*******************/     /*******************/     /*******************/     /***/"the calm value, zero.",)]     fn choices(size:u32,t     :u32)->u64{u64::from(     /**/t.min(size-t))+1}
    /***/#[doc=concat!(" Every edge of the ","flake: the twelve ","copies of one ","half-arm.",)]fn edges(&self)->BTreeSet<Edge>{self.arm().into_iter().flat_map(Edge::orbit).collect()}#[doc=concat!(" Half of one arm: the ",
  "spine heading east ","and the side branches ","on its",)]#[doc=concat!(" north side. The rest ","of the flake is ","copies of this.",)]fn arm(&self)->BTreeSet<Edge>{let size=self.weather.len()as i32;let spine=Branch{at:Hex
 /*******/(0,0),dir:0,turn:1,born:0,len:size,strip:size,};let mut edges=BTreeSet::new();self.grow(spine,&mut edges);edges}#[doc=concat!(" Lay down `branch` ","and grow its side ","branches. Whenever ","the weather is",)]#[doc=
/*********/concat!(" nonzero for the time ","a step is reached, a ","side branch starts ","there.",)]#[doc=""]#[doc=concat!(" A point's time is ","the same as its ","distance from the ","centre, so one weather",)]#[doc=concat!(
/******************/" value puts side ","branches on a whole ","hexagonal ring. Real ","snow crystals have",)]#[doc=concat!(" the same kind of ","banding.",)]#[doc=""]#[doc=concat!(" A side branch is ","never longer than its ",
/**********/"distance to either ","end of its",)]#[doc=concat!(" parent. That keeps ","it inside its half of ","the 60° wedge, so the ","copies of",)]#[doc=concat!(" the arm don't ","overlap, and it makes ","branches shorter ",
 /*********************************/"toward the tip.",)]#[doc=""]#[doc=concat!(" Side branches also ","split the space ","beside their parent ","into strips, one",)]#[doc=concat!(" per branch, running ","from that branch to ",
  /****************************/"the next one along. A ","branch's",)]#[doc=concat!(" own side branches ","can't be longer than ","its strip is wide. ","Since no",)]#[doc=concat!(" branch reaches into ","a sibling's strip, ",
    /**************************************************/"branches never share ","an edge, and",)]#[doc=concat!(" each side branch off ","the spine is drawn at ","exactly the length the",)]#[doc=concat!(" weather gave it. ",
                    /**/"That makes the "     /******************/,     /*******************/     /****/"weather readable back ",     "from the flake, so",     /***/)]#[doc=concat!(     " different weather "
                   /******************/,     /**/"always gives a "     ,"different flake.",)     /**/]fn grow(&self,branch:Branch,     edges: &mut BTreeSet<     /*****/Edge>){let mut     next=branch.len;for i
                  in(0..branch.len).rev     /***/(){let p=branch.     at.step(branch.dir,i)     /*****/;edges.insert(Edge::new(p,p.     step(branch.dir,1)));     /****/let reach=self.     weather[(branch.born+
                 /**/i)as usize].min(i     /**/).min(branch.len-     i).min(branch.strip);     if reach>0{self.grow(branch.sprout(i,     reach,next-i),edges);     /**/next=i;}}}}#[doc=     /***********/concat!(
                /***/" Draws ASCII by ","default, or Unicode ",     /***/"with the alternate ","flag (`{:#}`).",)]impl fmt::Display for Snowflake{fn fmt(&self,     /********/f: &mut fmt::Formatter)->fmt::Result{
               /***************/f.write_str(&self.render(if f.     /****************************/alternate(){Style::UNICODE}else{Style::ASCII}))}}#[doc=concat!(     /**/" The glyphs a flake ","is drawn with.",)]#
              /***/[derive(Clone,Copy,Debug,PartialEq,Eq)]pub     /****/struct Style{pub flat:char,pub rise:char, /*****************/pub fall:char,#[doc=concat!(     " Used where lines in ","different directions "
             /*****/,"meet.",)]pub knot:char,}impl Style{pub     /***/const ASCII:Style=Style{flat:'-',rise:'/',   /***************/fall:'\\',knot:'*',};pub const     UNICODE:Style=Style{flat:'─',rise:'╱',fall:'╲',
             /**************/knot:'•',};fn glyph(self,axes:      /****/Option<&u8>)->char{match axes{None=>' ',     /********/Some(0b001)=>self.rise,Some(0b010)=>      /*****/self.flat,Some(0b100)=>self.fall,Some(_
             /*************/)=>self.knot,}}}#[doc=concat!(       " One straight branch ","of an arm.",)]#[derive   /************/(Clone,Copy)]struct Branch{#[doc=       /**********/" Where it starts."]at:Hex,#[doc=
              /*********/concat!(" Which way it grows, ",         /**/"in sixths of a turn ","counter-clockwise " /******/,"from east.",)]dir:usize,#[doc=concat!         /*******************/(" How far its side ",
               /*****************/"branches turn from ",           /**************/"it, in sixths of a ","turn. Theirs",)]#[doc=concat!(" turn back the other ",           /**************/"way, which keeps them ",
                 /****/"parallel to this ","branch.",)]              turn:usize,#[doc=concat!(" The time it starts. ","Time goes up by one ","with each step."              /**********/,)]born:i32,#[doc=concat!(
                                 /*******************/                               " How many steps long ","it is.",)]len:i32,#[doc=concat!(                               /*******************/
                                /*******************/                               /************/" How wide its strip ","is, which caps how ",                               /**/"long its side ",
                               "branches can be.",)]                               strip:i32,}impl Branch{#[doc=concat!(" The side branch that "                               ,"starts `i` steps ",
                              /**/"along this one."                               /**/,)]fn sprout(&self,i:i32,len:i32,strip:i32)->Branch{Branch{                               at:self.at.step(self.
                             /******************************/dir,i),dir:(self.dir+self.turn)%6,turn:6-self.turn,born:self.born+i,len,strip,}}}#[doc=concat!(" A point on the ","triangular lattice, ",
                            /*****************************/"in axial coordinates.",)]#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]struct Hex(i32,i32);impl Hex{#[doc=concat!(" Unit steps, ",
                           "counter-clockwise ","from east.",)]const DIRS:[Hex;6]=[Hex(1,0),Hex(1,-1 /**/),Hex(0,-1),Hex(-1,0) /**************/,Hex(-1,1),Hex(0,1),];fn step(self,dir:usize,n:i32)->Hex{
                          /***********/let Hex(dq,dr)=Hex::DIRS[dir];Hex(self.0+n*dq,self.1+n*dr)}#   /********/[doc=concat!(   /***********/" Rotate 60° ","counter-clockwise ","about the origin.",)]fn
                          /***********/rotate(self)->Hex{Hex(self.0+self.1,-self.0)}#[doc=concat!(     " Mirror across the "     /****************/,"east-west axis.",)]fn reflect(self)->Hex{Hex(self.0+
                          /******************/self.1,-self.1)}#[doc=concat!(" The character cell ",     /*****************/     /**/"(column, row) for ","this point. One step ","east is two columns.",)
                           /***/]fn screen(self)->(i32,i32){(2*self.0+self.1,self.1)}}#[doc=concat!(     /***************/     " An undirected unit ","edge between ","neighbouring lattice ","points.",
                            /**************/)]#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]     struct Edge(Hex     /*************/,Hex);impl Edge{fn new(a:Hex,b:Hex)->Edge{Edge(a.min(b),a.
                              /****************/max(b))}fn map(self,f:fn(Hex)->Hex)->Edge{Edge::new(f(      /**/self.0)      /***********************/,f(self.1))}#[doc=concat!(" The 12 copies of ",
                                              /*******************/     /******/"this edge under the ",                     /******/"hexagon's rotations ",     "and reflections.",)]
                                             /***/fn orbit(self)->     /******/impl Iterator<Item=Edge>{                   /**/successors(Some(self),|e|Some     (e.map(Hex::rotate)))
                                            .take(6).flat_map(|e|     /**/[e,e.map(Hex::reflect)])}}#[cfg                 (test)]mod tests{use super::*;#[doc     /**********/=concat!(
                                           /*******************/     /***********/" Flip the art top to ",               /**************/"bottom or left to ",     /*******************/
                                          /**/"right, swapping / and ","\\ to match.",)]fn mirror(art: &str             /**/,vertical:bool)->String{let width=art.lines().map(|l|l.chars(
                                         ).count()).max().unwrap_or(0);let swap= |c|match c{'/'=>'\\','\\'=>           /***/'/',c=>c,};let rows=art.lines().map(|l|format!("{l:width$}"));
                                        /**/let rows:Vec<String> =if vertical{rows.rev( /**/).map(|l|l.chars(         ).map(swap).collect() /**/).collect()}else{rows.map(|l|l.chars().rev(
                                       /**/).map(swap).collect()).collect()};rows.iter   ().map(|l|l.trim_end(       )).collect::<Vec<_>>(   /***********************/).join("\n")}#[test]fn
                                       /**/mirror_symmetric(){for id in(0..64).map(|k     /***/|k*225){let art=     Snowflake::from_id(9,     /**/id).unwrap().to_string();assert_eq!(mirror
                                       /*******/(&art,true),art,"id {id}");assert_eq!(   mirror(&art,false),art,   /******/"id {id}");}}#[   /*********/test]fn arm_stays_in_its_wedge(){for
                                        /*******/id in 0..Snowflake::count(8).unwrap(){ /****/let arm=Snowflake:: /**/from_id(8,id).unwrap( /******/).arm();for Hex(q,r)in arm.into_iter().
                                         /*******************************************************/flat_map(|Edge(a,b)|[a,b]){assert!(r<=0&&q+2*r>=0,"id {id}: {:?}",Hex(q,r));}}}#[test]fn
                                           /****/every_id_is_a_different_flake(){for size in 1..=8{let count=Snowflake::count(size).unwrap();let flakes:BTreeSet<_> =(0..count).map(|id|
                                                           /***************/Snowflake::from_id(size,id).unwrap().edges()).collect();assert_eq!(flakes.len()as u64,count,
                                                          /***********************************/"size {size}");assert_eq!(Snowflake::from_id(size,count),None);}}#[test]fn
                                                         /*******/ids_past_max_size_cover_the_whole_flake(){let size=Snowflake::MAX_ID_SIZE+1;assert_eq!(Snowflake::count(
                                                        /*********************/size-1),Some(2_982_752_926_433_280_000));assert_eq!(Snowflake::count(size),None);assert_eq!(
                                                       /*******************/Snowflake::from_id(size,0).unwrap().weather[24],0);assert_eq!(Snowflake::from_id(size,u64::MAX).
                                                      /**********************/unwrap().weather[24],1);let ends=(0..256).chain(u64::MAX-255..=u64::MAX);let flakes:BTreeSet<_>
                                                     /*****/=ends.map(|id| /****/Snowflake::from_id( size,id).unwrap().edges() /***********/).collect(); assert_eq!(flakes.len
                                                    /***/(),512);}#[test]   fn group_closes(){let h   /**********/=Hex(3,-1);   /**********/assert_eq!(   successors(Some(h),|h
                                                    /**/|Some(h.rotate()     /**/)).nth(6),Some(h)     /****/);assert_eq!(h.     reflect().reflect(),h     /**************/);}}
                                                    /*****************/       /*****************/       /*****************/       /*****************/       /*****************/
                                                     /***************/         /***************/         /***************/         /***************/         /***************/
                                                      /*************/           /*************/           /*************/           /*************/           /*************/
                                                        /*********/               /*********/               /*********/               /*********/               /*********/
