insert into RustersDb.Clearances
	(
		Sequence, Name
	)
select 0, 'King'
union all
select 1, 'Queen'
union all
select 2, 'Rook'
union all
select 3, 'Bishop'
union all
select 4, 'Knight'
union all
select 5, 'Pawn';
