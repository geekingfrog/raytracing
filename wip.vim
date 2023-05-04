let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
silent only
silent tabonly
cd ~/dev/rust/raytracing
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
if &shortmess =~ 'A'
  set shortmess=aoOA
else
  set shortmess=aoO
endif
badd +259 src/main.rs
badd +189 health://
badd +1 Cargo.toml
badd +48 ~/.cargo/registry/src/github.com-1ecc6299db9ec823/epaint-0.21.0/src/image.rs
argglobal
%argdel
$argadd src/main.rs
edit src/main.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 90 + 88) / 177)
exe 'vert 2resize ' . ((&columns * 86 + 88) / 177)
argglobal
balt ~/.cargo/registry/src/github.com-1ecc6299db9ec823/epaint-0.21.0/src/image.rs
setlocal fdm=indent
setlocal fde=
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=999
setlocal fml=1
setlocal fdn=20
setlocal fen
2
normal! zo
40
normal! zo
41
normal! zo
54
normal! zo
55
normal! zo
62
normal! zo
64
normal! zo
64
normal! zc
62
normal! zc
55
normal! zc
54
normal! zc
41
normal! zc
40
normal! zc
122
normal! zo
201
normal! zo
202
normal! zo
205
normal! zo
216
normal! zo
308
normal! zo
309
normal! zo
311
normal! zo
312
normal! zo
313
normal! zo
330
normal! zo
let s:l = 206 - ((19 * winheight(0) + 17) / 35)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 206
normal! 026|
wincmd w
argglobal
if bufexists(fnamemodify("src/main.rs", ":p")) | buffer src/main.rs | else | edit src/main.rs | endif
if &buftype ==# 'terminal'
  silent file src/main.rs
endif
balt ~/.cargo/registry/src/github.com-1ecc6299db9ec823/epaint-0.21.0/src/image.rs
setlocal fdm=manual
setlocal fde=
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 259 - ((24 * winheight(0) + 17) / 35)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 259
normal! 026|
wincmd w
2wincmd w
exe 'vert 1resize ' . ((&columns * 90 + 88) / 177)
exe 'vert 2resize ' . ((&columns * 86 + 88) / 177)
tabnext 1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let &winminheight = s:save_winminheight
let &winminwidth = s:save_winminwidth
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
set hlsearch
nohlsearch
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
